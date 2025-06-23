//! # Stratum Core Crate
//!
//! `stratum_core` is the umbrella crate for all building blocks of the SRI project.

#[cfg(feature = "with_network_helpers")]
pub use network_helpers_sv2;
pub use roles_logic_sv2;
#[cfg(feature = "with_key_utils")]
pub use key_utils;
#[cfg(feature = "with_sv1")]
pub use sv1_api;