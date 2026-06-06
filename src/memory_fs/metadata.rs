use crate::{
    domain::{BlockId, InodeId},
    storage::{BLOCK_SIZE, Block},
};

pub struct FsMetadata {
    pub magic_number: u64,

    pub total_inodes: u64,
    pub free_inodes: u64,

    pub total_blocks: u64,
    pub free_blocks: u64,

    pub root_inode: InodeId,
    pub inode_table_start: BlockId,
    pub inode_table_blocks: u64,
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

impl FsMetadata {
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

impl TryFrom<Block> for FsMetadata {
    type Error = InvalidSuperBlockError;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        let magic_number = read_u64_at(&block.data, MAGIC_NUMBER_FIELD)?;

        if magic_number != FS_MAGIC {
            return Err(InvalidSuperBlockError);
        }

        let metadata = FsMetadata {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{BLOCK_SIZE, Block};

    fn valid_metadata() -> FsMetadata {
        FsMetadata {
            magic_number: FS_MAGIC,

            total_inodes: 100,
            free_inodes: 10,

            total_blocks: 100,
            free_blocks: 10,

            root_inode: 7,
            inode_table_start: 2,
            inode_table_blocks: 3,
        }
    }

    fn write_u64_at(buffer: &mut [u8], value: u64, field_index: usize) {
        let start = field_index * U64_SIZE;
        let end = start + U64_SIZE;

        let dst = buffer
            .get_mut(start..end)
            .expect("expected space in the buffer");

        dst.copy_from_slice(&value.to_be_bytes());
    }

    fn valid_superblock() -> Block {
        let mut data = [0u8; BLOCK_SIZE];

        write_u64_at(&mut data, FS_MAGIC, MAGIC_NUMBER_FIELD);

        write_u64_at(&mut data, 100, TOTAL_INODES_FIELD);
        write_u64_at(&mut data, 10, FREE_INODES_FIELD);

        write_u64_at(&mut data, 100, TOTAL_BLOCKS_FIELD);
        write_u64_at(&mut data, 10, FREE_BLOCKS_FIELD);

        write_u64_at(&mut data, 7, ROOT_INODE_FIELD);
        write_u64_at(&mut data, 2, INODE_TABLE_START_FIELD);
        write_u64_at(&mut data, 3, INODE_TABLE_BLOCKS_FIELD);

        Block { id: 0, data }
    }

    #[test]
    fn valid_metadata_passes_validation() {
        let metadata = valid_metadata();

        assert!(metadata.validate().is_ok());
    }

    #[test]
    fn invalid_magic_number_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.magic_number = 123;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn zero_total_inodes_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_inodes = 0;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn zero_total_blocks_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_blocks = 0;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn free_inodes_greater_than_total_inodes_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_inodes = 100;
        metadata.free_inodes = 101;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn free_blocks_greater_than_total_blocks_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_blocks = 100;
        metadata.free_blocks = 101;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn root_inode_equal_to_total_inodes_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_inodes = 100;
        metadata.root_inode = 100;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn root_inode_greater_than_total_inodes_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_inodes = 100;
        metadata.root_inode = 101;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn inode_table_start_equal_to_total_blocks_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_blocks = 100;
        metadata.inode_table_start = 100;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn inode_table_start_greater_than_total_blocks_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.total_blocks = 100;
        metadata.inode_table_start = 101;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn zero_inode_table_blocks_fails_validation() {
        let mut metadata = valid_metadata();
        metadata.inode_table_blocks = 0;

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn try_from_valid_superblock_returns_metadata() {
        let block = valid_superblock();

        let metadata = FsMetadata::try_from(block).expect("valid superblock should parse");

        assert_eq!(metadata.magic_number, FS_MAGIC);
        assert_eq!(metadata.total_inodes, 100);
        assert_eq!(metadata.free_inodes, 10);
        assert_eq!(metadata.total_blocks, 100);
        assert_eq!(metadata.free_blocks, 10);
        assert_eq!(metadata.root_inode, 7);
        assert_eq!(metadata.inode_table_start, 2);
        assert_eq!(metadata.inode_table_blocks, 3);
    }

    #[test]
    fn try_from_rejects_superblock_with_invalid_magic_number() {
        let mut block = valid_superblock();

        write_u64_at(&mut block.data, 123, MAGIC_NUMBER_FIELD);

        assert!(FsMetadata::try_from(block).is_err());
    }

    #[test]
    fn try_from_rejects_superblock_with_invalid_free_inodes() {
        let mut block = valid_superblock();

        write_u64_at(&mut block.data, 101, FREE_INODES_FIELD);

        assert!(FsMetadata::try_from(block).is_err());
    }

    #[test]
    fn try_from_rejects_superblock_with_invalid_free_blocks() {
        let mut block = valid_superblock();

        write_u64_at(&mut block.data, 101, FREE_BLOCKS_FIELD);

        assert!(FsMetadata::try_from(block).is_err());
    }

    #[test]
    fn to_block_rejects_invalid_metadata() {
        let mut metadata = valid_metadata();
        metadata.root_inode = metadata.total_inodes;

        assert!(metadata.to_block(0).is_err());
    }

    #[test]
    fn to_block_serializes_valid_metadata() {
        let metadata = valid_metadata();

        let block = metadata
            .to_block(0)
            .expect("valid metadata should serialize");

        assert_eq!(
            read_u64_at(&block.data, MAGIC_NUMBER_FIELD).unwrap(),
            FS_MAGIC
        );
        assert_eq!(read_u64_at(&block.data, TOTAL_INODES_FIELD).unwrap(), 100);
        assert_eq!(read_u64_at(&block.data, FREE_INODES_FIELD).unwrap(), 10);
        assert_eq!(read_u64_at(&block.data, TOTAL_BLOCKS_FIELD).unwrap(), 100);
        assert_eq!(read_u64_at(&block.data, FREE_BLOCKS_FIELD).unwrap(), 10);
        assert_eq!(read_u64_at(&block.data, ROOT_INODE_FIELD).unwrap(), 7);
        assert_eq!(
            read_u64_at(&block.data, INODE_TABLE_START_FIELD).unwrap(),
            2
        );
        assert_eq!(
            read_u64_at(&block.data, INODE_TABLE_BLOCKS_FIELD).unwrap(),
            3
        );
    }

    #[test]
    fn metadata_round_trip_through_block() {
        let metadata = valid_metadata();

        let block = metadata.to_block(0).expect("metadata should serialize");
        let parsed = FsMetadata::try_from(block).expect("metadata should deserialize");

        assert_eq!(parsed.magic_number, metadata.magic_number);
        assert_eq!(parsed.total_inodes, metadata.total_inodes);
        assert_eq!(parsed.free_inodes, metadata.free_inodes);
        assert_eq!(parsed.total_blocks, metadata.total_blocks);
        assert_eq!(parsed.free_blocks, metadata.free_blocks);
        assert_eq!(parsed.root_inode, metadata.root_inode);
        assert_eq!(parsed.inode_table_start, metadata.inode_table_start);
        assert_eq!(parsed.inode_table_blocks, metadata.inode_table_blocks);
    }
}
