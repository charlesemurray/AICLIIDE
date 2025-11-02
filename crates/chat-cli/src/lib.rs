//! This lib.rs is only here for testing purposes.
//! `test_mcp_server/test_server.rs` is declared as a separate binary and would need a way to
//! reference types defined inside of this crate, hence the export.
pub mod analytics;
pub mod api_client;
pub mod auth;
pub mod aws_common;
pub mod cli;
pub mod constants;
pub mod database;
pub mod git;
pub mod logging;
pub mod mcp_client;
pub mod os;
pub mod request;
pub mod session;
pub mod telemetry;
pub mod theme;
pub mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert_eq!(2 + 2, 4);
    }
}

pub use mcp_client::*;
