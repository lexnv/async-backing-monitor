use codec::Encode;

use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "./artifacts/asset_hub_kusama.scale")]
pub mod asset_hub_kusama {}

#[subxt::subxt(runtime_metadata_path = "./artifacts/kusama.scale")]
pub mod kusama_relay_chain {}

#[tokio::main]
pub async fn main() {
    // Reconnect on loop errors.
    loop {
        if let Err(err) = run().await {
            eprintln!("{err}");
            println!("ERROR: {err}");
        }
    }
}

type ParaInclusionEvent =
    kusama_relay_chain::runtime_types::polkadot_runtime_parachains::inclusion::pallet::Event;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let api =
        OnlineClient::<PolkadotConfig>::from_url("wss://statemine.public.curie.radiumblock.co/ws")
            .await?;
    println!("Connection with parachain established.");

    let kusama_api =
        OnlineClient::<PolkadotConfig>::from_url("wss://rpc-kusama.helixstreet.io").await?;

    println!("Connection with Kusama relay chain established.");

    let mut parachain_sub = api.blocks().subscribe_best().await?;

    let mut relay_chain_sub = kusama_api.blocks().subscribe_best().await?;

    let mut timestamps = std::collections::HashMap::new();

    let mut relay_chain_time = std::time::Instant::now();
    let mut now = std::time::Instant::now();
    let mut duplicated_blocks = std::collections::HashMap::new();
    let mut last_author = None;

    loop {
        tokio::select! {
            block = relay_chain_sub.next() => {
                let Some(block) = block else {
                    break;
                };
                let block = block?;

                let block_number = block.header().number;

                println!("  Relay Block #{block_number}, hash={:?} (elasped {:?})", block.hash(), relay_chain_time.elapsed());
                relay_chain_time = std::time::Instant::now();

                // Log each of the extrinsic with it's associated events:
                let extrinsics = block.extrinsics().await?;
                for ext in extrinsics.iter() {
                    let events = ext.events().await?;

                    for evt in events.iter() {
                        let evt = evt?;
                        let Ok(event) = evt.as_root_event::<kusama_relay_chain::Event>() else {
                            continue;
                        };

                        match event {
                            kusama_relay_chain::Event::ParaInclusion(ParaInclusionEvent::CandidateBacked(receipt, ..)) => {
                                let descriptor = receipt.descriptor;
                                let para_id = descriptor.para_id.0;
                                let relay_chain_parent = descriptor.relay_parent;
                                let para_head = descriptor.para_head;

                                if para_id != 1000 {
                                    continue;
                                }
                                println!("   |--> CandidateBacked: para_head={:?} relay_parent={:?}\n", para_head, relay_chain_parent);
                            }
                            _ => (),
                        };
                    }
                }
            },

            block = parachain_sub.next() => {
                let Some(block) = block else {
                    break;
                };
                let block = block?;
                let block_number = block.header().number;

                if block.header().digest.logs.is_empty() {
                    println!("  No logs in this block.");
                    continue;
                }

                let author = &block.header().digest.logs[0];

                let extrinsics = block
                    .extrinsics()
                    .await
                    .inspect_err(|err| println!("Failed to decode extrinsics: {:?}", err))?;


                let mut timestamp = None;
                let mut duplicate = None;

                for ext in extrinsics.iter() {
                    let decoded_ext = ext.as_root_extrinsic::<asset_hub_kusama::Call>();
                    match decoded_ext {
                        Ok(asset_hub_kusama::Call::Timestamp(asset_hub_kusama::runtime_types::pallet_timestamp::pallet::Call::set {now})) => {
                            timestamp = Some(now);

                             match timestamps.entry(now) {
                                std::collections::hash_map::Entry::Occupied(mut entry) => {
                                    let block = entry.get_mut();
                                    duplicated_blocks.insert(*block, block_number);
                                    duplicate = Some((*block, block_number));
                                }
                                std::collections::hash_map::Entry::Vacant(entry) => {
                                    entry.insert(block_number);
                                }
                            }
                        }
                        _ => {}

                    }
                }

                let author_bytes = author.encode();
                let same_author = last_author.as_ref().map(|last| last == &author_bytes).unwrap_or(false);
                let author_labe = if same_author { "Same" } else { "New" };
                last_author = Some(author_bytes);

                if let Some((_origin_block, duplicate_number)) = duplicate {
                    println!("[X] AssetHubKusama: Block #{block_number}, hash={:?} (elasped {:?})", block.hash(), now.elapsed());
                    println!("  |--> {author_labe} Author: {:?}", hex::encode(author.encode()));
                    println!("  |--> ({}) Duplicate Timestamp extrinsic found: initial={} current_block={}\n", duplicated_blocks.len(), duplicate_number, block_number);
                } else {
                    println!("AssetHubKusama: Block #{block_number}, hash={:?} (elasped {:?})", block.hash(), now.elapsed());
                    println!("  |--> {author_labe} Author: {:?}", hex::encode(author.encode()));
                    println!("  |--> Timestamp.Set: {:?}\n", timestamp.unwrap_or_default());
                }

                now = std::time::Instant::now();
            }
        }
    }

    println!(
        "Total duplicated timestamps found: {}.",
        duplicated_blocks.len()
    );

    Ok(())
}
