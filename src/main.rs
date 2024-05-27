use tokio::sync::mpsc;

mod peer;
mod types;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("RUST_LOG")
        .init();

    let mut peer = peer::Peer::new("127.0.0.1:18444").await.unwrap();
    let commands = peer.add_command_sender();

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        peer.add_listener(tx);
        match peer.start().await {
            Ok(_) => log::info!("Peer thread finished"),
            Err(e) => log::error!("Peer thread failed: {}", e),
        }
    });

    commands
        .send(types::Command::get_blocks(Vec::new()))
        .await
        .unwrap();

    while let Some(message) = rx.recv().await {
        log::info!("⚡️ {:?}", message);
    }
}
