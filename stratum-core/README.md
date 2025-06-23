# `stratum_core`

[![crates.io](https://img.shields.io/crates/v/stratum_core.svg)](https://crates.io/crates/stratum_core)
[![docs.rs](https://docs.rs/stratum_core/badge.svg)](https://docs.rs/stratum_core)
[![rustc+](https://img.shields.io/badge/rustc-1.75.0%2B-lightgrey.svg)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)
[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/stratum-mining/stratum/blob/main/LICENSE.md)
[![codecov](https://codecov.io/gh/stratum-mining/stratum/branch/main/graph/badge.svg?flag=buffer_sv2-coverage)](https://codecov.io/gh/stratum-mining/stratum)

`stratum_core` is the umbrella crate for all building blocks of the SRI project.

It re-exports the following crates:
- `key-utils`: with primitives to serialize and deserialize `secp256k1` keys. Optionally re-exported via `with_key_utils` feature flag.
- `network_helpers_sv2`: with high-level networking abstractions for building Sv2 and Sv1 apps. Optionally re-exported via `with_network_helpers` feature flag.
- `sv1_api`: with primitives for Stratum V1. Optionally re-exported via `with_sv1` feature flag.
- `roles_logic_sv2`: containing high-level primitives for application development, which also re-exports the following crates:
  - `bitcoin` (from its own `rust-bitcoin` dependency)
  - `binary_sv2`: with primitives for serializing and deserializing Sv2 types
  - `framing_sv2`: with primitives for creating Sv2 frames
  - `codec_sv2`: with primitives for encoding and decoding Sv2 frames (with and without encryption) to be sent over the network
  - `noise_sv2`: with primitives for Sv2 noise encryption
  - `common_messages_sv2`: with message abstractions that are common to all Sv2 applications (regardless of subprotocol)
  - `mining_sv2`: with message abstractions under the Sv2 mining subprotocol
  - `job_declaration_sv2`: with message abstractions under the Sv2 job declaration subprotocol
  - `template_distribution_sv2`: with message abstractions under the Sv2 template distribution subprotocol