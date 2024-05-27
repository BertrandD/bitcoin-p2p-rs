use bitcoin::{blockdata::block::Block, hashes::Hash, p2p::message_blockdata, BlockHash};

#[derive(Debug)]
pub enum Event {
    NewBlock(Block),
}

#[derive(Debug)]
pub struct GetBlocksCommand {
    pub known_blocks: Vec<BlockHash>,
    pub stop_hash: BlockHash,
}

impl GetBlocksCommand {
    pub fn as_message(&self) -> message_blockdata::GetBlocksMessage {
        message_blockdata::GetBlocksMessage {
            version: bitcoin::p2p::PROTOCOL_VERSION,
            locator_hashes: self.known_blocks.clone(),
            stop_hash: self.stop_hash,
        }
    }
}

#[derive(Debug)]
pub enum Command {
    GetBlocks(GetBlocksCommand),
}

impl Command {
    pub fn get_blocks(known_blocks: Vec<BlockHash>) -> Command {
        Command::GetBlocks(GetBlocksCommand {
            known_blocks,
            stop_hash: BlockHash::all_zeros(),
        })
    }
}
