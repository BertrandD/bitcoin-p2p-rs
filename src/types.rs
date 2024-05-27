use bitcoin::blockdata::block::Block;

#[derive(Debug)]
pub enum Event {
    NewBlock(Block),
}
