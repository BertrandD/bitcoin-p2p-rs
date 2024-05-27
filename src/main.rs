mod peer;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("RUST_LOG")
        .init();

    let peer = peer::Peer::new("127.00.1:18444").await;
    match peer {
        Ok(mut p) => p.start().await.unwrap(),
        Err(e) => {
            log::error!("Error creating peer: {:?}", e);
        }
    }
}
