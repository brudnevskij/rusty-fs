use crate::{
    domain::{BlockId, InodeId},
    storage::{BLOCK_SIZE, Block},
};

pub struct FsMetadata {
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

    use crate::{
        memory_fs::metadata::{
            self, FREE_BLOCKS_FIELD, FREE_INODES_FIELD, FS_MAGIC, FsMetadata,
            INODE_TABLE_BLOCKS_FIELD, INODE_TABLE_START_FIELD, MAGIC_NUMBER_FIELD,
            ROOT_INODE_FIELD, TOTAL_BLOCKS_FIELD, TOTAL_INODES_FIELD, U64_SIZE,
        },
        storage::Block,
    };

    // tests helper
    fn write_u64_at(buffer: &mut [u8], value: u64, field_index: usize, field_size: usize) {
        let start = field_index * field_size;
        let end = start + field_size;
        let dst = buffer
            .get_mut(start..end)
            .expect("expecting space in the buffer");
        dst.copy_from_slice(&value.to_be_bytes());
    }

    fn get_correct_super_block() -> Block {
        let mut data = [0; _];

        write_u64_at(&mut data, FS_MAGIC, MAGIC_NUMBER_FIELD, U64_SIZE);

        write_u64_at(&mut data, 100, TOTAL_INODES_FIELD, U64_SIZE);
        write_u64_at(&mut data, 10, FREE_INODES_FIELD, U64_SIZE);

        write_u64_at(&mut data, 100, TOTAL_BLOCKS_FIELD, U64_SIZE);
        write_u64_at(&mut data, 10, FREE_BLOCKS_FIELD, U64_SIZE);

        write_u64_at(&mut data, 7, ROOT_INODE_FIELD, U64_SIZE);
        write_u64_at(&mut data, 2, INODE_TABLE_START_FIELD, U64_SIZE);
        write_u64_at(&mut data, 3, INODE_TABLE_BLOCKS_FIELD, U64_SIZE);

        Block { id: 0, data }
    }

    #[test]
    fn correct_metadata_init() {
        let super_block = get_correct_super_block();
        let metadata: FsMetadata = super_block.try_into().expect("correct init");

        assert_eq!(metadata.magic_number, FS_MAGIC);
        assert_eq!(metadata.total_inodes, 100);
        assert_eq!(metadata.free_inodes, 10);
        assert_eq!(metadata.total_blocks, 100);
        assert_eq!(metadata.free_blocks, 10);
        assert_eq!(metadata.root_inode, 7);
        assert_eq!(metadata.inode_table_start, 2);
        assert_eq!(metadata.inode_table_blocks, 3);
    }
}
