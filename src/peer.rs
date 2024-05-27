use bitcoin::consensus::encode::Error;
use bitcoin::consensus::Decodable;
use bitcoin::consensus::Encodable;
use bitcoin::hashes::Hash;
use bitcoin::io::Cursor;
use bitcoin::io::ErrorKind;
use bitcoin::p2p::message::{NetworkMessage, RawNetworkMessage};
use bitcoin::p2p::message_blockdata;
use bitcoin::p2p::message_network::VersionMessage;
use bitcoin::p2p::{Magic, ServiceFlags};
use bitcoin::BlockHash;
use std::io;
use std::io::Write;
use std::net::SocketAddr;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncWriteExt, Interest};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::types::Command;
use crate::types::Event;

#[derive(Debug)]
pub enum PeerError {
    // /// Can't reach the peer.
    PeerUnreachable(io::Error),
}

pub struct Peer {
    stream: TcpStream,
    listeners: Vec<mpsc::Sender<Event>>,
    commands: mpsc::Receiver<Command>,
    command_sender: mpsc::Sender<Command>,
}

impl Peer {
    pub async fn new(addr: &str) -> Result<Peer, PeerError> {
        let stream = Peer::connect_tcp(addr).await?;
        let (tx, rx) = mpsc::channel(100);

        Ok(Peer {
            stream,
            listeners: Vec::new(),
            commands: rx,
            command_sender: tx,
        })
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.handshake().await;

        let mut buf = Vec::with_capacity(1024 * 1024);

        loop {
            let ready = self
                .stream
                .ready(Interest::READABLE | Interest::WRITABLE | Interest::ERROR)
                .await?;

            if ready.is_error() {
                panic!("Error on stream");
            }

            match self.stream.read_buf(&mut buf).await {
                Ok(0) => {
                    log::info!("Connection closed by remote");
                    return Ok(());
                }
                Ok(_) => {
                    let messages = Peer::handle_read(buf.len(), &mut buf).await;
                    for msg in messages {
                        self.handle_message(msg).await;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }

            if let Ok(command) = self.commands.try_recv() {
                match command {
                    Command::GetBlocks(cmd) => {
                        self.send_message(NetworkMessage::GetBlocks(cmd.as_message()))
                            .await;
                    }
                }
            }
        }
    }

    pub fn add_command_sender(&self) -> mpsc::Sender<Command> {
        self.command_sender.clone()
    }

    pub fn add_listener(&mut self, tx: mpsc::Sender<Event>) {
        self.listeners.push(tx);
    }

    async fn connect_tcp(addr: &str) -> Result<TcpStream, PeerError> {
        match TcpStream::connect(addr).await {
            Ok(conn) => {
                log::info!("Connected to peer: {}", addr);
                Ok(conn)
            }
            Err(e) => {
                log::error!("Failed to connect to peer: {}", e);
                Err(PeerError::PeerUnreachable(e))
            }
        }
    }

    fn make_version_msg(
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

    async fn handle_message(&mut self, msg: RawNetworkMessage) {
        log::debug!("RECV: {}", msg.cmd());
        match msg.payload() {
            NetworkMessage::Version(_) => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                self.send_message(NetworkMessage::Verack).await;
            }
            NetworkMessage::Verack => {}
            NetworkMessage::Alert(data) => {
                std::str::from_utf8(data)
                    .map(|s| log::info!("Alert message: {}", s))
                    .unwrap_or_else(|_| {
                        log::error!("Failed to decode alert message: {:?}", data);
                    });
            }
            NetworkMessage::Ping(n) => self.pong(*n).await,
            NetworkMessage::Inv(inv) => self.inventory(inv).await,
            NetworkMessage::Block(block) => {
                log::debug!("Block data: {:?}", block);
                for tx in self.listeners.iter() {
                    tx.send(Event::NewBlock(block.clone())).await.unwrap();
                }
            }
            _ => log::warn!("Unknown message: {:?}", msg),
        }
    }

    async fn send_message(&mut self, msg: NetworkMessage) {
        log::debug!("SEND: {}", msg.cmd());
        let raw_msg = RawNetworkMessage::new(Magic::REGTEST, msg);
        let mut buf = std::io::BufWriter::new(Vec::new());
        raw_msg.consensus_encode(&mut buf).unwrap();
        buf.flush().unwrap();
        self.stream.write_all(buf.get_ref()).await.unwrap();
    }

    async fn inventory(&mut self, inv: &[message_blockdata::Inventory]) {
        self.send_message(NetworkMessage::GetData(inv.to_vec()))
            .await;
        // for i in inv {
        //     match i {
        //         bitcoin::p2p::message_blockdata::Inventory::Block(hash) => {
        //             log::debug!("Block: {}", hash);
        //             self.get_data(*i).await;
        //         }
        //         bitcoin::p2p::message_blockdata::Inventory::Transaction(hash) => {
        //             log::debug!("Transaction: {}", hash);
        //         }
        //         bitcoin::p2p::message_blockdata::Inventory::CompactBlock(hash) => {
        //             log::debug!("CompactBlock: {}", hash);
        //         }
        //         bitcoin::p2p::message_blockdata::Inventory::WTx(id) => log::info!("WTx: {}", id),
        //         bitcoin::p2p::message_blockdata::Inventory::WitnessTransaction(id) => {
        //             log::debug!("WitnessTransaction: {}", id)
        //         }
        //         bitcoin::p2p::message_blockdata::Inventory::WitnessBlock(hash) => {
        //             log::debug!("WitnessBlock: {}", hash)
        //         }
        //         _ => log::warn!("Unknown inventory type"),
        //     }
        // }
    }

    async fn handshake(&mut self) {
        let addr = self.stream.peer_addr().unwrap();
        let loc_addr = self.stream.local_addr().unwrap();
        let block_height = 0;
        let version_msg =
            NetworkMessage::Version(Peer::make_version_msg(addr, loc_addr, block_height));
        self.send_message(version_msg).await;
    }

    async fn pong(&mut self, n: u64) {
        self.send_message(NetworkMessage::Pong(n)).await;
    }

    async fn handle_read(n: usize, buf: &mut Vec<u8>) -> Vec<RawNetworkMessage> {
        let mut v = &buf[..n];
        let mut cursor = Cursor::new(&mut v);
        let mut messages = Vec::new();
        let mut pos = cursor.position() as usize;
        loop {
            let raw_msg = match RawNetworkMessage::consensus_decode(&mut cursor) {
                Ok(msg) => {
                    pos = cursor.position() as usize;
                    msg
                }
                Err(Error::Io(ref e)) if e.kind() == ErrorKind::UnexpectedEof => {
                    log::debug!("Missing end of message in buffer, waiting for more data");
                    break;
                }
                Err(e) => {
                    log::error!("Failed to decode message: {}", e);
                    break;
                }
            };
            messages.push(raw_msg);

            if pos == n {
                break;
            }
        }

        buf.drain(..pos);
        messages
    }
}
