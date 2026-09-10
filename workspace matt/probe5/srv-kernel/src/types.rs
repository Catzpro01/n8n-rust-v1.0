use serde::{Deserialize, Serialize};

/// Status eksekusi workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Waiting,
    Success,
    Failed,
    NeedsReview,
    Canceled,
}

/// Status task individual
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Ready,
    Running,
    Waiting,
    Success,
    Failed,
    InDoubt,
    Canceled,
}

/// Jenis side effect suatu node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideEffect {
    None,
    Idempotent,
    NonIdempotent,
}

/// Resource hint untuk scheduler/governor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceHint {
    pub side_effect: SideEffect,
    pub weight: Option<u32>,
    pub max_concurrency: Option<u16>,
}

/// Descriptor node — sumber tunggal informasi tentang suatu node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub kind: String,
    pub version: u32,
    pub display_name: String,
    pub group: Vec<String>,
    pub hints: ResourceHint,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub execute_once: bool,
}
