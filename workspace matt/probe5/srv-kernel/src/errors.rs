use thiserror::Error;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("parameter validation failed: {0}")]
    Validation(String),
    #[error("execution failed: {0}")]
    Execution(String),
    #[error("credential error: {0}")]
    Credential(String),
    #[error("timeout")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("workflow not found: {0}")]
    WorkflowNotFound(String),
    #[error("invalid workflow: {0}")]
    InvalidWorkflow(String),
    #[error("storage error: {0}")]
    Storage(String),
}
