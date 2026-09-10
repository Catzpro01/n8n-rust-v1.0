//! Error types for EBC

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EbcError {
    #[error("QuickJS compilation failed: {0}")]
    Compilation(String),
    
    #[error("QuickJS execution failed: {0}")]
    Execution(String),
    
    #[error("Cache storage error: {0}")]
    Storage(String),
    
    #[error("Bytecode deserialization failed: {0}")]
    Deserialization(String),
    
    #[error("Expression validation failed: {0}")]
    Validation(String),
}
