use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "./artifacts/asset_hub_kusama.scale")]
pub mod asset_hub_kusama {}

#[tokio::main]
pub async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        println!("ERROR: {err}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let api =
        OnlineClient::<PolkadotConfig>::from_url("wss://statemine.public.curie.radiumblock.co/ws")
            .await?;
    println!("Connection with parachain established.");

    let mut parachain_sub = api.blocks().subscribe_best().await?;
    let mut timestamps = std::collections::HashMap::new();

    let mut now = std::time::Instant::now();
    while let Some(block) = parachain_sub.next().await {
        let block = block.inspect_err(|err| println!("Failed receiving block {:?}", err))?;

        let block_number = block.header().number;

        println!("Block #{block_number}:");
        println!("  hash={:?} (elasped {:?})", block.hash(), now.elapsed());

        if block.header().digest.logs.is_empty() {
            println!("  No logs in this block.");
            continue;
        }

        let author = &block.header().digest.logs[0];
        println!("  Author: {:?}", author);

        let mut index = 0;
        let extrinsics = block
            .extrinsics()
            .await
            .inspect_err(|err| println!("Failed to decode extrinsics: {:?}", err))?;
        for ext in extrinsics.iter() {
            index += 1;

            if index == 2 {
                let bytes = ext.bytes().to_vec();

                match timestamps.entry(bytes) {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        let block = entry.get_mut();
                        println!(
                            " [x] Duplicate Timestamp extrinsic found: initial={} current_block={}",
                            block, block_number
                        );
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(block_number);
                    }
                }
                println!(
                    "    Timestamp.Set #{}, Bytes: {:?}\n",
                    ext.index(),
                    ext.bytes()
                );
            }
        }

        now = std::time::Instant::now();
    }

    Ok(())
}
