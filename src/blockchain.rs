use std::collections::HashMap;

use bitcoin::{blockdata::block::Block as BlockData, BlockHash};

#[derive(Clone, Debug)]
pub struct Block {
    data: BlockData,
    height: u64,
}

pub struct Blockchain {
    blocks_by_hash: HashMap<BlockHash, Block>,
    blocks_by_height: Vec<Block>,
    height: u64,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks_by_hash: HashMap::new(),
            blocks_by_height: Vec::new(),
            height: 0,
        }
    }

    pub fn add_block(&mut self, block: BlockData) {
        if self.blocks_by_height.is_empty() {
            log::debug!(
                "Adding first block: {} with genesis parent {}",
                block.header.block_hash(),
                block.header.prev_blockhash
            );
            self.blocks_by_hash.insert(
                block.header.block_hash(),
                Block {
                    data: block,
                    height: 0,
                },
            );
        } else if let Some(b) = self.blocks_by_hash.get(&block.header.prev_blockhash) {
            log::debug!(
                "Adding block: {} with parent {}, height {}",
                block.header.block_hash(),
                block.header.prev_blockhash,
                b.height + 1
            );
            self.blocks_by_hash.insert(
                block.header.block_hash(),
                Block {
                    data: block,
                    height: b.height + 1,
                },
            );
        } else {
            log::warn!(
                "Block with unknown parent: {:?}. Ignoring.",
                block.header.prev_blockhash
            );
        }
    }

    pub fn commit(&mut self) {
        // TODO: handle reorgs !
        self.blocks_by_height = self.blocks_by_hash.values().cloned().collect();
        self.blocks_by_height.sort_by_key(|block| block.height);
        self.height = self.blocks_by_height.len() as u64;
    }

    pub fn len(&self) -> usize {
        self.blocks_by_height.len()
    }

    pub fn last_blocks(&self, count: usize) -> Vec<BlockHash> {
        self.blocks_by_height
            .iter()
            .rev()
            .take(count)
            .rev()
            .map(|block| block.data.block_hash())
            .collect()
    }

    pub fn get_block_by_height(&self, height: usize) -> Option<&Block> {
        self.blocks_by_height.get(height)
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        // self.blocks_by_hash.get(hash)
        todo!()
    }
}
