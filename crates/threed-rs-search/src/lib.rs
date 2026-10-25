//! Spatial search and neighborhood indexing for 3D point clouds.
//!
//! Powered by Kiddo's SIMD-accelerated k-d trees and Rayon parallelism.

pub mod error;
pub mod index;
pub mod query;

// Re-exports
pub use error::{Result, SearchError};
pub use index::KdTreeIndex;
pub use query::{NeighborItem, NeighborhoodMethod};
