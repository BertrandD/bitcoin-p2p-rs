use std::collections::HashMap;

use bitcoin::blockdata::transaction::{Transaction, Txid};

pub struct Mempool {
    mempool: HashMap<Txid, Transaction>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            mempool: HashMap::new(),
        }
    }
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.mempool.insert(tx.compute_txid(), tx);
    }

    pub fn size(&self) -> u64 {
        self.mempool.len() as u64
    }
}
