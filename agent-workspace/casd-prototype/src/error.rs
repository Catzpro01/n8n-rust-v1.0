use thiserror::Error;

#[derive(Error, Debug)]
pub enum CasdError {
    #[error("Storage I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Index error: {0}")]
    Index(String),
    
    #[error("Hash collision detected: hash={hash}, existing_size={existing}, new_size={new}")]
    Collision { hash: String, existing: u64, new: u64 },
    
    #[error("Content not found: {0}")]
    NotFound(String),
    
    #[error("Disk full: required {required} bytes, available {available} bytes")]
    DiskFull { required: u64, available: u64 },
}
