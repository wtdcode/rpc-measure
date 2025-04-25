use alloy::providers::{Provider, ProviderBuilder};
use chrono::{DateTime, Utc};
use clap::Parser;
use color_eyre::Result;
use incr_stats::incr::Stats;
use tokio_stream::StreamExt;

#[derive(Parser)]
struct RpcArgs {
    #[arg(short, long)]
    pub rpc: String,
}

async fn measure(args: RpcArgs) -> Result<()> {
    let rpc = ProviderBuilder::new().connect(&args.rpc).await?;

    let mut sub = rpc.subscribe_blocks().await?.into_stream();
    let mut s = Stats::new();

    println!("Start to listen blocks...");
    while let Some(hdr) = sub.next().await {
        let now = Utc::now();

        let ts = DateTime::from_timestamp(hdr.timestamp as i64, 0).unwrap();
        let diff = (now - ts).num_milliseconds();
        s.update(diff as f64)?;
        println!(
            "Block {} arrived, current: {:.02} ms, avg: {:.02} ms, std: {:.02}, max: {:.02} ms, min: {:.02} ms",
            hdr.number,
            diff,
            s.mean().unwrap(),
            s.sample_standard_deviation().unwrap_or_default(),
            s.max().unwrap(),
            s.min().unwrap()
        );
    }

    Ok(())
}

fn main() {
    color_eyre::install().unwrap();
    let args = RpcArgs::parse();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(measure(args))
        .expect("??");
}
