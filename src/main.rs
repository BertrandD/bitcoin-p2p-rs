use bitcoin::consensus::Decodable;
use bitcoin::consensus::Encodable;
use bitcoin::io::Cursor;
use bitcoin::io::ErrorKind;
use bitcoin::p2p::message::{NetworkMessage, RawNetworkMessage};
use bitcoin::p2p::message_network::VersionMessage;
use bitcoin::p2p::{Magic, ServiceFlags};
use std::io;
use std::io::Write;
use std::net::SocketAddr;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum Error {
    // /// Can't reach the peer.
    PeerUnreachable(io::Error),
}

async fn connect_tcp(addr: &str) -> Result<TcpStream, Error> {
    match TcpStream::connect(addr).await {
        Ok(conn) => {
            log::info!("Connected to peer: {}", addr);
            Ok(conn)
        }
        Err(e) => {
            log::error!("Failed to connect to peer: {}", e);
            Err(Error::PeerUnreachable(e))
        }
    }
}

pub fn make_version_msg(
    addr: SocketAddr,
    local_addr: SocketAddr,
    start_height: u32,
) -> VersionMessage {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let msg = VersionMessage {
        version: bitcoin::p2p::PROTOCOL_VERSION,
        services: ServiceFlags::NONE,
        timestamp,
        receiver: bitcoin::p2p::address::Address::new(&addr, ServiceFlags::NONE),
        sender: bitcoin::p2p::address::Address::new(&local_addr, ServiceFlags::NONE),
        nonce: rand::random(),
        user_agent: "bitcoin-p2p-rs".parse().unwrap(),
        start_height: start_height as i32,
        relay: false,
    };

    log::info!("Prepared version message ({}): {:?}", addr, msg);
    msg
}

async fn handle_message(stream: &mut TcpStream, msg: RawNetworkMessage) {
    log::info!("RECV: {}", msg.cmd());
    match msg.payload() {
        NetworkMessage::Version(_) => {
            tokio::time::sleep(Duration::from_millis(50)).await;
            send_message(stream, NetworkMessage::Verack).await;
        }
        NetworkMessage::Verack => {}
        NetworkMessage::Alert(data) => {
            std::str::from_utf8(data)
                .map(|s| log::info!("Alert message: {}", s))
                .unwrap_or_else(|_| {
                    log::error!("Failed to decode alert message: {:?}", data);
                });
        }
        NetworkMessage::Ping(n) => pong(stream, *n).await,
        NetworkMessage::Inv(inv) => inventory(stream, inv).await,
        NetworkMessage::Block(block) => log::info!("Block data: {:?}", block),
        _ => log::warn!("Unknown message: {:?}", msg),
    }
}

async fn send_message(stream: &mut TcpStream, msg: NetworkMessage) {
    log::info!("SEND: {}", msg.cmd());
    let raw_msg = RawNetworkMessage::new(Magic::REGTEST, msg);
    let mut buf = std::io::BufWriter::new(Vec::new());
    raw_msg.consensus_encode(&mut buf).unwrap();
    buf.flush().unwrap();
    stream.write_all(buf.get_ref()).await.unwrap();
}

async fn get_data(stream: &mut TcpStream, data: bitcoin::p2p::message_blockdata::Inventory) {
    send_message(stream, NetworkMessage::GetData(vec![data])).await;
}

async fn inventory(stream: &mut TcpStream, inv: &Vec<bitcoin::p2p::message_blockdata::Inventory>) {
    for i in inv {
        match i {
            bitcoin::p2p::message_blockdata::Inventory::Block(hash) => {
                log::info!("Block: {}", hash);
                get_data(stream, *i).await;
            }
            bitcoin::p2p::message_blockdata::Inventory::Transaction(hash) => {
                log::info!("Transaction: {}", hash);
            }
            bitcoin::p2p::message_blockdata::Inventory::CompactBlock(hash) => {
                log::info!("CompactBlock: {}", hash);
            }
            bitcoin::p2p::message_blockdata::Inventory::WTx(id) => log::info!("WTx: {}", id),
            bitcoin::p2p::message_blockdata::Inventory::WitnessTransaction(id) => {
                log::info!("WitnessTransaction: {}", id)
            }
            bitcoin::p2p::message_blockdata::Inventory::WitnessBlock(hash) => {
                log::info!("WitnessBlock: {}", hash)
            }
            _ => log::warn!("Unknown inventory type"),
        }
    }
}

async fn handshake(stream: &mut TcpStream) {
    let addr = stream.peer_addr().unwrap();
    let loc_addr = stream.local_addr().unwrap();
    let block_height = 0;
    let version_msg = NetworkMessage::Version(make_version_msg(addr, loc_addr, block_height));
    send_message(stream, version_msg).await;
}

async fn pong(stream: &mut TcpStream, n: u64) {
    send_message(stream, NetworkMessage::Pong(n)).await;
}

async fn handle_read(stream: &mut TcpStream, n: usize, buf: Vec<u8>) {
    let mut v = &buf[..n];
    let mut cursor = Cursor::new(&mut v);
    loop {
        // let pos = cursor.position() as usize;
        let raw_msg = match RawNetworkMessage::consensus_decode(&mut cursor) {
            Ok(msg) => msg,
            Err(bitcoin::consensus::encode::Error::Io(ref e))
                if e.kind() == ErrorKind::UnexpectedEof =>
            {
                log::error!("Buffer too small to read message: {}", e);
                break;
            }
            Err(e) => {
                log::error!("Failed to decode message: {}", e);
                break;
            }
        };
        handle_message(stream, raw_msg).await;

        if cursor.position() as usize == n {
            break;
        }
    }
    match RawNetworkMessage::consensus_decode(&mut v) {
        Ok(msg) => handle_message(stream, msg).await,
        Err(e) => log::error!("Failed to decode message: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_env("RUST_LOG")
        .init();

    log::info!("Hello, world!");

    let mut stream = connect_tcp("127.0.0.1:18444").await.unwrap();

    handshake(&mut stream).await;

    loop {
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE | Interest::ERROR)
            .await?;

        if ready.is_error() {
            panic!("Error on stream");
        }

        if ready.is_readable() {
            let mut buf = Vec::with_capacity(4096);
            match stream.try_read_buf(&mut buf) {
                Ok(0) => {
                    log::info!("Connection closed by remote");
                    return Ok(());
                }
                Ok(n) => {
                    log::debug!("read {} bytes", n);
                    handle_read(&mut stream, n, buf).await;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}
