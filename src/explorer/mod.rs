//! Block Explorer for RoboChain
//!
//! Provides a REST API and web interface for browsing the blockchain,
//! including blocks, transactions, and domain objects (robots, spaces, tasks).

mod types;
mod service;
mod api;

pub use types::*;
pub use service::ExplorerService;
pub use api::create_router;
