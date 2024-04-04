use roles_logic_sv2::{
    parsers::Mining,
};
use std::{fmt, sync::PoisonError};
use v1::server_to_client::SetDifficulty;

use stratum_common::bitcoin::util::uint::ParseLengthError;

pub type TProxyResult<'a, T> = core::result::Result<T, TProxyError<'a>>;

#[derive(Debug)]
pub enum TProxyError<'a> {
    VecToSlice32(Vec<u8>),
    /// Errors on bad CLI argument input.
    BadCliArgs,
    /// Errors on bad `serde_json` serialize/deserialize.
    BadSerdeJson(serde_json::Error),
    /// Errors on bad `toml` deserialize.
    BadTomlDeserialize(toml::de::Error),
    /// Errors from `binary_sv2` crate.
    BinarySv2(binary_sv2::Error),
    /// Errors on bad noise handshake.
    CodecNoise(codec_sv2::noise_sv2::Error),
    /// Errors from `framing_sv2` crate.
    FramingSv2(framing_sv2::Error),
    /// Errors on bad `TcpStream` connection.
    Io(std::io::Error),
    /// Errors on bad `String` to `int` conversion.
    ParseInt(std::num::ParseIntError),
    /// Errors from `roles_logic_sv2` crate.
    RolesSv2Logic(roles_logic_sv2::errors::Error),
    UpstreamIncoming(roles_logic_sv2::errors::Error),
    /// SV1 protocol library error
    V1Protocol(v1::error::Error<'a>),
    #[allow(dead_code)]
    SubprotocolMining(String),
    // Locking Errors
    PoisonLock,
    // Channel Receiver Error
    ChannelErrorReceiver(async_channel::RecvError),
    TokioChannelErrorRecv(tokio::sync::broadcast::error::RecvError),
    Uint256Conversion(ParseLengthError),
    SetDifficultyToMessage(SetDifficulty),
    Infallible(std::convert::Infallible),
    // used to handle SV2 protocol error messages from pool
    #[allow(clippy::enum_variant_names)]
    Sv2ProtocolError(Mining<'a>),
}

impl<'a> fmt::Display for TProxyError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TProxyError::*;
        match self {
            BadCliArgs => write!(f, "Bad CLI arg input"),
            BadSerdeJson(ref e) => write!(f, "Bad serde json: `{:?}`", e),
            BadTomlDeserialize(ref e) => write!(f, "Bad `toml` deserialize: `{:?}`", e),
            BinarySv2(ref e) => write!(f, "Binary SV2 error: `{:?}`", e),
            CodecNoise(ref e) => write!(f, "Noise error: `{:?}", e),
            FramingSv2(ref e) => write!(f, "Framing SV2 error: `{:?}`", e),
            Io(ref e) => write!(f, "I/O error: `{:?}", e),
            ParseInt(ref e) => write!(f, "Bad convert from `String` to `int`: `{:?}`", e),
            RolesSv2Logic(ref e) => write!(f, "Roles SV2 Logic Error: `{:?}`", e),
            V1Protocol(ref e) => write!(f, "V1 Protocol Error: `{:?}`", e),
            SubprotocolMining(ref e) => write!(f, "Subprotocol Mining Error: `{:?}`", e),
            UpstreamIncoming(ref e) => write!(f, "Upstream parse incoming error: `{:?}`", e),
            PoisonLock => write!(f, "Poison Lock error"),
            ChannelErrorReceiver(ref e) => write!(f, "Channel receive error: `{:?}`", e),
            TokioChannelErrorRecv(ref e) => write!(f, "Channel receive error: `{:?}`", e),
            Uint256Conversion(ref e) => write!(f, "U256 Conversion Error: `{:?}`", e),
            SetDifficultyToMessage(ref e) => {
                write!(f, "Error converting SetDifficulty to Message: `{:?}`", e)
            }
            VecToSlice32(ref e) => write!(f, "Standard Error: `{:?}`", e),
            Infallible(ref e) => write!(f, "Infallible Error:`{:?}`", e),
            Sv2ProtocolError(ref e) => {
                write!(f, "Received Sv2 Protocol Error from upstream: `{:?}`", e)
            }
        }
    }
}

impl<'a> From<binary_sv2::Error> for TProxyError<'a> {
    fn from(e: binary_sv2::Error) -> Self {
        TProxyError::BinarySv2(e)
    }
}

impl<'a> From<codec_sv2::noise_sv2::Error> for TProxyError<'a> {
    fn from(e: codec_sv2::noise_sv2::Error) -> Self {
        TProxyError::CodecNoise(e)
    }
}

impl<'a> From<framing_sv2::Error> for TProxyError<'a> {
    fn from(e: framing_sv2::Error) -> Self {
        TProxyError::FramingSv2(e)
    }
}

impl<'a> From<std::io::Error> for TProxyError<'a> {
    fn from(e: std::io::Error) -> Self {
        TProxyError::Io(e)
    }
}

impl<'a> From<std::num::ParseIntError> for TProxyError<'a> {
    fn from(e: std::num::ParseIntError) -> Self {
        TProxyError::ParseInt(e)
    }
}

impl<'a> From<roles_logic_sv2::errors::Error> for TProxyError<'a> {
    fn from(e: roles_logic_sv2::errors::Error) -> Self {
        TProxyError::RolesSv2Logic(e)
    }
}

impl<'a> From<serde_json::Error> for TProxyError<'a> {
    fn from(e: serde_json::Error) -> Self {
        TProxyError::BadSerdeJson(e)
    }
}

impl<'a> From<toml::de::Error> for TProxyError<'a> {
    fn from(e: toml::de::Error) -> Self {
        TProxyError::BadTomlDeserialize(e)
    }
}

impl<'a> From<v1::error::Error<'a>> for TProxyError<'a> {
    fn from(e: v1::error::Error<'a>) -> Self {
        TProxyError::V1Protocol(e)
    }
}

impl<'a> From<async_channel::RecvError> for TProxyError<'a> {
    fn from(e: async_channel::RecvError) -> Self {
        TProxyError::ChannelErrorReceiver(e)
    }
}

impl<'a> From<tokio::sync::broadcast::error::RecvError> for TProxyError<'a> {
    fn from(e: tokio::sync::broadcast::error::RecvError) -> Self {
        TProxyError::TokioChannelErrorRecv(e)
    }
}

//*** LOCK ERRORS ***
impl<'a, T> From<PoisonError<T>> for TProxyError<'a> {
    fn from(_e: PoisonError<T>) -> Self {
        TProxyError::PoisonLock
    }
}

impl<'a> From<Vec<u8>> for TProxyError<'a> {
    fn from(e: Vec<u8>) -> Self {
        TProxyError::VecToSlice32(e)
    }
}

impl<'a> From<ParseLengthError> for TProxyError<'a> {
    fn from(e: ParseLengthError) -> Self {
        TProxyError::Uint256Conversion(e)
    }
}

impl<'a> From<SetDifficulty> for TProxyError<'a> {
    fn from(e: SetDifficulty) -> Self {
        TProxyError::SetDifficultyToMessage(e)
    }
}

impl<'a> From<std::convert::Infallible> for TProxyError<'a> {
    fn from(e: std::convert::Infallible) -> Self {
        TProxyError::Infallible(e)
    }
}

impl<'a> From<Mining<'a>> for TProxyError<'a> {
    fn from(e: Mining<'a>) -> Self {
        TProxyError::Sv2ProtocolError(e)
    }
}
