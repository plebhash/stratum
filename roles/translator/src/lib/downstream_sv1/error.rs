
pub type TProxyDownstreamResult<'a, T> = Result<T, TProxyDownstreamError<'a>>;
pub enum TProxyDownstreamError<'a> {
    PoisonLock,
    BinarySv2(binary_sv2::Error),
    Send(async_channel::SendError<v1::Message>),
    V1(v1::error::Error<'a>),
    V1MessageTooLong,
    ParseLength(stratum_common::bitcoin::util::uint::ParseLengthError),
    Target(roles_logic_sv2::errors::Error),
    ConnectionAborted,
    BadSerdeJson(serde_json::Error),
    ChannelReceiver(async_channel::RecvError),
    TokioChannelErrorRecv(tokio::sync::broadcast::error::RecvError),
    Io(std::io::Error),
}

impl<'a> From<binary_sv2::Error> for TProxyDownstreamError<'a> {
    fn from(e: binary_sv2::Error) -> TProxyDownstreamError<'a> {
        TProxyDownstreamError::BinarySv2(e)
    }
}

impl<'a> From<async_channel::RecvError> for TProxyDownstreamError<'a> {
    fn from(e: async_channel::RecvError) -> Self {
        TProxyDownstreamError::ChannelReceiver(e)
    }
}

impl<'a> From<std::io::Error> for TProxyDownstreamError<'a> {
    fn from(e: std::io::Error) -> Self {
        TProxyDownstreamError::Io(e)
    }
}

impl<'a> From<tokio::sync::broadcast::error::RecvError> for TProxyDownstreamError<'a> {
    fn from(e: tokio::sync::broadcast::error::RecvError) -> Self {
        TProxyDownstreamError::TokioChannelErrorRecv(e)
    }
}

impl<'a> From<serde_json::Error> for TProxyDownstreamError<'a> {
    fn from(e: serde_json::Error) -> Self {
        TProxyDownstreamError::BadSerdeJson(e)
    }
}

impl<'a> From<v1::error::Error<'a>> for TProxyDownstreamError<'a> {
    fn from(e: v1::error::Error<'a>) -> TProxyDownstreamError {
        TProxyDownstreamError::V1(e)
    }
}

impl<'a> From<stratum_common::bitcoin::util::uint::ParseLengthError> for TProxyDownstreamError<'a> {
    fn from(e: stratum_common::bitcoin::util::uint::ParseLengthError) -> TProxyDownstreamError<'a> {
        TProxyDownstreamError::ParseLength(e)
    }
}

impl<'a> From<roles_logic_sv2::errors::Error> for TProxyDownstreamError<'a> {
    fn from(e: roles_logic_sv2::errors::Error) -> TProxyDownstreamError<'a> {
        TProxyDownstreamError::Target(e)
    }
}