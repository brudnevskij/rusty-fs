use crate::{domain::BlockId, error::DiskResult};

pub const BLOCK_SIZE: usize = 4096;

pub struct Block {
    id: BlockId,
    data: [u8; BLOCK_SIZE],
}

pub trait Disk {
    fn read_block(&self, block_id: BlockId) -> DiskResult<Block>;
    fn write_block(&mut self, block_id: BlockId, data: &Block) -> DiskResult<()>;
}
