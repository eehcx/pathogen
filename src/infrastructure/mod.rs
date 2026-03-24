// Implementations for external systems
// Examples: MockFirewallRepository, CliFirewallRepository

pub mod audit_logger;
pub mod mock_repository;
pub mod cli_repository;
pub mod nftables_json;

pub use cli_repository::CliFirewallRepository;
