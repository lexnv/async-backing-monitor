use clap::Parser as ClapParser;
use codec::Encode;
use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "./artifacts/asset_hub_kusama.scale")]
pub mod asset_hub_kusama {}

#[subxt::subxt(runtime_metadata_path = "./artifacts/kusama.scale")]
pub mod kusama_relay_chain {}

type ParaInclusionEvent =
    kusama_relay_chain::runtime_types::polkadot_runtime_parachains::inclusion::pallet::Event;

/// Command for interacting with the CLI.
#[derive(Debug, ClapParser)]
enum Command {
    /// Subscribe to the parachain and relay chain blocks.
    ///
    /// This command will connect to the specified relay chain and parachain URLs,
    /// and will continuously monitor for new blocks, printing out the block number,
    /// hash, author, and timestamp of each block (and warn on duplicated timestamps).
    Subscribe {
        #[clap(long, default_value = "wss://rpc-kusama.helixstreet.io")]
        relay_chain_url: String,

        #[clap(long, default_value = "wss://asset-hub-kusama.dotters.network")]
        parachain_url: String,
    },

    /// Archive mode to fetch and print blocks from the parachain.
    ///
    /// This command connects to the specified parachain URL and retrieves
    /// blocks within a specified range (default is 200 blocks back from the latest).
    Archive {
        #[clap(long, default_value = "wss://asset-hub-kusama.dotters.network")]
        parachain_url: String,

        #[clap(long)]
        blocks_diff: Option<u32>,
    },
}

#[tokio::main]
pub async fn main() {
    let args = Command::parse();

    match args {
        Command::Subscribe {
            relay_chain_url,
            parachain_url,
        } => {
            // Reconnect on loop errors.
            loop {
                if let Err(err) = AsyncBackingMonitor::new()
                    .run(relay_chain_url.as_str(), parachain_url.as_str())
                    .await
                {
                    eprintln!("{err}");
                    println!("ERROR: {err}");
                }
            }
        }
        Command::Archive {
            parachain_url,
            blocks_diff,
        } => {
            archive(parachain_url.as_str(), blocks_diff.unwrap_or(200))
                .await
                .expect("Failed to run archive mode");
        }
    }
}

async fn archive(parachain_url: &str, blocks_diff: u32) -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(parachain_url).await?;
    println!("Connection with parachain established.");

    let rpc_client = subxt_rpcs::RpcClient::from_url(parachain_url).await?;
    let legacy_methods: subxt_rpcs::LegacyRpcMethods<PolkadotConfig> =
        subxt_rpcs::LegacyRpcMethods::new(rpc_client);
    println!("Connection with RPC client established.");

    let latest = api.blocks().at_latest().await?;
    let number = latest.header().number;
    println!(
        "AssetHubKusama: Latest parachain block #{number}, hash={:?}",
        latest.hash()
    );

    let mut target = number - blocks_diff;
    let mut timestamps = std::collections::HashMap::new();
    let mut duplicated_blocks = std::collections::HashMap::new();
    let mut last_author = None;

    let mut authoring_statistincs = std::collections::HashMap::new();
    let mut authoring_in_row = std::collections::HashMap::new();
    let mut num_produced = 1;

    while target != number {
        let hash = legacy_methods
            .chain_get_block_hash(Some(target.into()))
            .await?
            .unwrap();
        let block = api.blocks().at(hash).await?;
        let block_number = block.header().number;

        target += 1;

        if block.header().digest.logs.is_empty() {
            println!("  No logs in this block.");
            return Ok(());
        }

        let author = &block.header().digest.logs[0];

        let extrinsics = block
            .extrinsics()
            .await
            .inspect_err(|err| println!("Failed to decode extrinsics: {:?}", err))?;

        let mut timestamp = None;
        let mut duplicate = None;

        let ext = extrinsics.iter().skip(1).next();
        if let Some(ext) = ext {
            let bytes = ext.bytes().to_vec();
            timestamp = Some(bytes.clone());

            match timestamps.entry(bytes) {
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

        let author_bytes = author.encode();
        authoring_statistincs
            .entry(author_bytes.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);

        let same_author = last_author
            .as_ref()
            .map(|last| last == &author_bytes)
            .unwrap_or(false);
        if same_author {
            num_produced += 1;
        } else {
            if num_produced > 1 {
                authoring_in_row
                    .entry(num_produced)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }

            num_produced = 1;
        }

        let author_labe = if same_author { "Same" } else { "New" };
        last_author = Some(author_bytes);

        if let Some((origin_block, _duplicate_number)) = duplicate {
            println!(
                "[X] AssetHubKusama: Block #{block_number}, hash={:?}",
                block.hash(),
            );
            println!(
                "  |--> {author_labe} Author: {:?}",
                hex::encode(author.encode())
            );
            println!(
                "  |--> ({}) Duplicate Timestamp extrinsic found: initial={} current_block={} Timestamp.Set: 0x{}\n",
                duplicated_blocks.len(),
                origin_block,
                block_number,
                hex::encode(timestamp.unwrap_or_default())
            );
        } else {
            println!(
                "AssetHubKusama: Block #{block_number}, hash={:?}",
                block.hash(),
            );
            println!(
                "  |--> {author_labe} Author: {:?}",
                hex::encode(author.encode())
            );
            println!(
                "  |--> Timestamp.Set: 0x{}\n",
                hex::encode(timestamp.unwrap_or_default())
            );
        }
    }

    println!("Archive completed successfully.");
    println!(
        "Total blocks with the same timestamp: {} / {} ({:.2}%)",
        duplicated_blocks.len(),
        blocks_diff,
        (duplicated_blocks.len() as f64 / blocks_diff as f64 * 100.0)
    );
    println!(" - produced in a row: {:#?}", authoring_in_row);

    Ok(())
}

struct AsyncBackingMonitor {
    timestamps: std::collections::HashMap<Vec<u8>, u32>,
    relay_chain_time: std::time::Instant,
    now: std::time::Instant,
    duplicated_blocks: std::collections::HashMap<u32, u32>,
    last_author: Option<Vec<u8>>,
}

impl AsyncBackingMonitor {
    fn new() -> Self {
        Self {
            timestamps: std::collections::HashMap::new(),
            relay_chain_time: std::time::Instant::now(),
            now: std::time::Instant::now(),
            duplicated_blocks: std::collections::HashMap::new(),
            last_author: None,
        }
    }

    async fn run(
        mut self,
        relay_chain_url: &str,
        parachain_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let api = OnlineClient::<PolkadotConfig>::from_url(parachain_url).await?;
        println!("Connection with parachain established.");

        let kusama_api = OnlineClient::<PolkadotConfig>::from_url(relay_chain_url).await?;
        println!("Connection with Kusama relay chain established.");

        let mut parachain_sub = api.blocks().subscribe_best().await?;
        let mut relay_chain_sub = kusama_api.blocks().subscribe_best().await?;

        loop {
            tokio::select! {
                block = relay_chain_sub.next() => {
                    let Some(block) = block else {
                        break;
                    };
                    let block = block?;

                    let block_number = block.header().number;
                    println!(
                        "  Relay Block #{block_number}, hash={:?} (elasped {:?})",
                        block.hash(),
                        self.relay_chain_time.elapsed()
                    );
                    self.relay_chain_time = std::time::Instant::now();

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
                                kusama_relay_chain::Event::ParaInclusion(
                                    ParaInclusionEvent::CandidateBacked(receipt, ..),
                                ) => {
                                    let descriptor = receipt.descriptor;
                                    let para_id = descriptor.para_id.0;
                                    let relay_chain_parent = descriptor.relay_parent;
                                    let para_head = descriptor.para_head;

                                    if para_id != 1000 {
                                        continue;
                                    }
                                    println!(
                                        "   |--> CandidateBacked: para_head={:?} relay_parent={:?}\n",
                                        para_head, relay_chain_parent
                                    );
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
                        return Ok(());
                    }

                    let author = &block.header().digest.logs[0];

                    let extrinsics = block
                        .extrinsics()
                        .await
                        .inspect_err(|err| println!("Failed to decode extrinsics: {:?}", err))?;

                    let mut timestamp = None;
                    let mut duplicate = None;

                    let ext = extrinsics.iter().skip(1).next();
                    if let Some(ext) = ext {
                        let bytes = ext.bytes().to_vec();
                        timestamp = Some(bytes.clone());

                        match self.timestamps.entry(bytes) {
                            std::collections::hash_map::Entry::Occupied(mut entry) => {
                                let block = entry.get_mut();
                                self.duplicated_blocks.insert(*block, block_number);
                                duplicate = Some((*block, block_number));
                            }
                            std::collections::hash_map::Entry::Vacant(entry) => {
                                entry.insert(block_number);
                            }
                        }
                    }

                    let author_bytes = author.encode();
                    let same_author = self
                        .last_author
                        .as_ref()
                        .map(|last| last == &author_bytes)
                        .unwrap_or(false);
                    let author_labe = if same_author { "Same" } else { "New" };
                    self.last_author = Some(author_bytes);

                    if let Some((origin_block, _duplicate_number)) = duplicate {
                        println!(
                            "[X] AssetHubKusama: Block #{block_number}, hash={:?} (elasped {:?})",
                            block.hash(),
                            self.now.elapsed()
                        );
                        println!(
                            "  |--> {author_labe} Author: {:?}",
                            hex::encode(author.encode())
                        );
                        println!(
                            "  |--> ({}) Duplicate Timestamp extrinsic found: initial={} current_block={} Timestamp.Set: 0x{}\n",
                            self.duplicated_blocks.len(),
                            origin_block,
                            block_number,
                            hex::encode(timestamp.unwrap_or_default())
                        );
                    } else {
                        println!(
                            "AssetHubKusama: Block #{block_number}, hash={:?} (elasped {:?})",
                            block.hash(),
                            self.now.elapsed()
                        );
                        println!(
                            "  |--> {author_labe} Author: {:?}",
                            hex::encode(author.encode())
                        );
                        println!(
                            "  |--> Timestamp.Set: 0x{}\n",
                            hex::encode(timestamp.unwrap_or_default())
                        );
                    }

                    self.now = std::time::Instant::now();
                }
            }
        }

        Ok(())
    }
}
