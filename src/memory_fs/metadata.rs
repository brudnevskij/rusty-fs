use crate::{
    domain::{BlockId, InodeId},
    storage::{BLOCK_SIZE, Block},
};

pub struct FsMetaData {
    magic_number: u64,

    total_inodes: u64,
    free_inodes: u64,

    total_blocks: u64,
    free_blocks: u64,

    root_inode: InodeId,
    inode_table_start: BlockId,
    inode_table_blocks: u64,
}

const U64_SIZE: usize = 8;

const MAGIC_NUMBER_FIELD: usize = 0;
const TOTAL_INODES_FIELD: usize = 1;
const FREE_INODES_FIELD: usize = 2;
const TOTAL_BLOCKS_FIELD: usize = 3;
const FREE_BLOCKS_FIELD: usize = 4;
const ROOT_INODE_FIELD: usize = 5;
const INODE_TABLE_START_FIELD: usize = 6;
const INODE_TABLE_BLOCKS_FIELD: usize = 7;

const SUPERBLOCK_FIELDS: usize = 8;
const SUPERBLOCK_SIZE: usize = SUPERBLOCK_FIELDS * U64_SIZE;

const FS_MAGIC: u64 = 0xF5F5_F00D_CAFE_BABE;

#[derive(Debug)]
pub struct InvalidSuperBlockError;

fn read_u64_at(data: &[u8], field: usize) -> Result<u64, InvalidSuperBlockError> {
    let start = field * 8;
    let end = start + 8;

    let bytes: [u8; 8] = data
        .get(start..end)
        .ok_or(InvalidSuperBlockError)?
        .try_into()
        .map_err(|_| InvalidSuperBlockError)?;

    Ok(u64::from_be_bytes(bytes))
}

fn write_u64_at(data: &mut [u8], field: usize, value: u64) -> Result<(), InvalidSuperBlockError> {
    let start = field * U64_SIZE;
    let end = start + U64_SIZE;

    let dst = data.get_mut(start..end).ok_or(InvalidSuperBlockError)?;

    dst.copy_from_slice(&value.to_be_bytes());

    Ok(())
}

impl FsMetaData {
    pub fn new(
        total_inodes: u64,
        total_blocks: u64,
        root_inode: InodeId,
        inode_table_start: BlockId,
        inode_table_blocks: u64,
    ) -> Self {
        Self {
            magic_number: FS_MAGIC,

            total_inodes,
            free_inodes: total_inodes,

            total_blocks,
            free_blocks: total_blocks,

            root_inode,
            inode_table_start,
            inode_table_blocks,
        }
    }

    pub fn to_block(&self, block_id: BlockId) -> Result<Block, InvalidSuperBlockError> {
        self.validate()?;

        let mut data = [0u8; BLOCK_SIZE];

        write_u64_at(&mut data, MAGIC_NUMBER_FIELD, self.magic_number)?;

        write_u64_at(&mut data, TOTAL_INODES_FIELD, self.total_inodes)?;
        write_u64_at(&mut data, FREE_INODES_FIELD, self.free_inodes)?;

        write_u64_at(&mut data, TOTAL_BLOCKS_FIELD, self.total_blocks)?;
        write_u64_at(&mut data, FREE_BLOCKS_FIELD, self.free_blocks)?;

        write_u64_at(&mut data, ROOT_INODE_FIELD, self.root_inode)?;
        write_u64_at(&mut data, INODE_TABLE_START_FIELD, self.inode_table_start)?;
        write_u64_at(&mut data, INODE_TABLE_BLOCKS_FIELD, self.inode_table_blocks)?;

        Ok(Block { id: block_id, data })
    }

    fn validate(&self) -> Result<(), InvalidSuperBlockError> {
        if self.magic_number != FS_MAGIC {
            return Err(InvalidSuperBlockError);
        }

        if self.total_inodes == 0 || self.total_blocks == 0 {
            return Err(InvalidSuperBlockError);
        }

        if self.free_inodes > self.total_inodes {
            return Err(InvalidSuperBlockError);
        }

        if self.free_blocks > self.total_blocks {
            return Err(InvalidSuperBlockError);
        }

        if self.root_inode >= self.total_inodes {
            return Err(InvalidSuperBlockError);
        }

        if self.inode_table_start >= self.total_blocks {
            return Err(InvalidSuperBlockError);
        }

        if self.inode_table_blocks == 0 {
            return Err(InvalidSuperBlockError);
        }

        Ok(())
    }
}

impl TryFrom<Block> for FsMetaData {
    type Error = InvalidSuperBlockError;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let magic_number = read_u64_at(&block.data, MAGIC_NUMBER_FIELD)?;

        if magic_number != FS_MAGIC {
            return Err(InvalidSuperBlockError);
        }

        let metadata = FsMetaData {
            magic_number,

            total_inodes: read_u64_at(&block.data, TOTAL_INODES_FIELD)?,
            free_inodes: read_u64_at(&block.data, FREE_INODES_FIELD)?,

            total_blocks: read_u64_at(&block.data, TOTAL_BLOCKS_FIELD)?,
            free_blocks: read_u64_at(&block.data, FREE_BLOCKS_FIELD)?,

            root_inode: read_u64_at(&block.data, ROOT_INODE_FIELD)?,
            inode_table_start: read_u64_at(&block.data, INODE_TABLE_START_FIELD)?,
            inode_table_blocks: read_u64_at(&block.data, INODE_TABLE_BLOCKS_FIELD)?,
        };

        metadata.validate()?;

        Ok(metadata)
    }
}
