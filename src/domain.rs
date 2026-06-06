pub type InodeId = u64;
pub type BlockId = u64;

pub enum FileType {
    File,
    Directory,
}

pub struct Metadata {
    pub file_type: FileType,
    pub size: u128,
}

pub enum Inode {
    File {
        id: InodeId,
        metadata: Metadata,
        data_pointers: Vec<BlockId>,
    },
    Directory {
        id: InodeId,
        directories: Vec<DirectoryEntry>,
    },
}

pub struct DirectoryEntry {
    pub name: String,
    pub inode_id: InodeId,
}
