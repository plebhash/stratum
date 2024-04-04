

pub type TProxyDownstreamResult<'a, T> = Result<T, TProxyDownstreamError<'a>>;

#[derive(Debug)]
pub enum TProxyDownstreamError<'a> {
    PoisonLock,
    BinarySv2(binary_sv2::Error),
    Send(async_channel::SendError<v1::Message>),
    V1(v1::error::Error<'a>),
    V1MessageTooLong,
    ParseLength(stratum_common::bitcoin::util::uint::ParseLengthError),
    Target(roles_logic_sv2::errors::Error),
    ConnectionAborted,
}

impl<'a> From<binary_sv2::Error> for TProxyDownstreamError<'a> {
    fn from(e: binary_sv2::Error) -> TProxyDownstreamError<'a> {
        TProxyDownstreamError::BinarySv2(e)
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