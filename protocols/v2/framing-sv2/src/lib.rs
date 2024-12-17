//! # Stratum V2 Framing Library
//!
//! `framing_sv2` provides utilities for framing messages sent between Sv2 roles, handling both Sv2
//! message and Noise handshake frames.
//!
//! ## Message Format
//!
//! The Sv2 protocol is binary, with fixed message framing. Each message begins with the extension
//! type, message type, and message length (six bytes in total), followed by a variable length
//! message.
//!
//! The message framing is outlined below ([according to Sv2 specs
//! ](https://stratumprotocol.org/specification/03-Protocol-Overview/#32-framing)):
//!
//! | Field Name | Byte Length | Description |
//! |----------------|-------------|-------------|
//! | `extension_type` | `U16` | Unique identifier of the extension associated with this protocol message. |
//! | `msg_type` | `U8` | Unique identifier of this protocol message. |
//! | `msg_length` | `U24` | Length of the protocol message, not including this header. |
//! | `payload` | `BYTES` | Message-specific payload of length `msg_length`. If the MSB in `extension_type` (the `channel_msg` bit) is set the first four bytes are defined as a `U32` `"channel_id"`, though this definition is repeated in the message definitions below and these 4 bytes are included in `msg_length`. |
//!
//! ## Usage
//!
//! Nearly all messages sent between Sv2 roles are serialized with the [`framing::Sv2Frame`]. The
//! exception is when two Sv2 roles exchange Noise protocol handshake messages.
//!
//! Before Sv2 roles can communicate securely, they must perform a Noise handshake (note that Noise
//! encryption is optional for communication between two local Sv2 roles (i.e. a local mining
//! device and a local mining proxy), but required between two remote Sv2 roles (i.e. a local
//! mining proxy and a remote pool)). During this process, the [`framing::HandShakeFrame`] is used
//! to transmit encrypted messages between the roles. After the handshake is completed and the
//! connection transitions into transport mode, [`framing::Sv2Frame`] is used for all messages.
//!
//! Once the Noise handshake is complete (if it was performed at all), all subsequent messages are
//! framed using the [`framing::Sv2Frame`]. Each frame consists of a [`header::Header`] followed by
//! a serialized payload.
//!
//! ## Build Options
//!
//! This crate can be built with the following features:
//!
//! - `with_buffer_pool`: Enables buffer pooling for more efficient memory management.
//! - `with_serde`: builds [`binary_sv2`] and [`buffer_sv2`](https://crates.io/crates/buffer_sv2)
//!   crates with `serde`-based encoding and decoding. Note that this feature flag is only used for
//!   the Message Generator, and deprecated for any other kind of usage. It will likely be fully
//!   deprecated in the future.
//!
//! ## Examples
//!
//! See the example for more information:
//!
//! - [Sv2 Frame Example](https://github.com/stratum-mining/stratum/blob/main/protocols/v2/framing-sv2/examples/sv2_frame.rs)

#![no_std]

extern crate alloc;

/// Sv2 framing types
pub mod framing;

/// Sv2 framing errors
pub mod error;

/// Sv2 framing header
pub mod header;
pub use error::Error;
