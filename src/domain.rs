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

pub struct Inode {
    pub id: InodeId,
    pub metadata: Metadata,
    pub data_pointers: Vec<BlockId>,
}

pub struct DirectoryEntry {
    pub name: String,
    pub inode_id: InodeId,
}
