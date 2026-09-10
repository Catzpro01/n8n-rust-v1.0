//! n8n-nodes-scrapling Prototype (W2-NODE-SCRAPLING)
//!
//! Adaptive stealth web scraping node with self-healing DOM parser.

pub mod error;
pub mod node;
pub mod output;
pub mod params;

pub use error::ScraplingError;
pub use node::ScraplingNode;
pub use output::ScraplingOutput;
pub use params::ScraplingParams;
