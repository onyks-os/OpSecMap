//! OpSecMap (OpSecMap)
//!
//! Local-first, zero-trust OpSec dependency mapper. Visualize accounts, devices,
//! backups and recovery paths to find single points of failure before an incident.
//!
//! Everything public in this crate is covered by semantic versioning; items that
//! are not exported here may change in any release.

/// The crate version, taken from `Cargo.toml` at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod model;

pub use model::{
    create_resource, DependencyMap, MapError, Resource, ResourceError, ResourceId, Route,
    RouteError, RouteId,
};

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_not_empty() {
        assert_ne!(VERSION, "");
    }
}
