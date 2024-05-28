use tokio::sync::mpsc;
use types::Command;

mod blockchain;
mod peer;
mod types;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
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

    let mut blockchain = blockchain::Blockchain::new();

    while let Some(message) = rx.recv().await {
        match message {
            types::Event::Connected => {
                commands
                    .send(Command::get_blocks(Vec::new()))
                    .await
                    .unwrap();
            }
            types::Event::NewBlock(block) => {
                blockchain.add_block(block);
            }
            types::Event::AllBlocksFetched => {
                let len_before = blockchain.len();
                blockchain.commit();
                if len_before != blockchain.len() {
                    log::info!("⛓️  Blockchain height: {}", blockchain.len());

                    let last_blocks = blockchain.last_blocks(10);
                    commands
                        .send(Command::get_blocks(last_blocks))
                        .await
                        .unwrap();
                }
            }
        }
    }
}
