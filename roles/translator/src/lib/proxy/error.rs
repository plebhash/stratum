use v1::server_to_client::Notify;

#[derive(Debug)]
pub enum BridgeChannelSendError<'a> {
    Notify(tokio::sync::broadcast::error::SendError<Notify<'a>>),
    SubmitSharesExtended(
        async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>,
    ),
}

pub type TProxyBridgeResult<'a, T> = Result<T, TProxyBridgeError<'a>>;

#[derive(Debug)]
pub enum TProxyBridgeError<'a> {
    PoisonLock,
    ChannelSender(BridgeChannelSendError<'a>),
    ChannelReceiver(async_channel::RecvError),
    RolesSv2Logic(roles_logic_sv2::errors::Error),
    SubprotocolMining(String),
    BinarySv2(binary_sv2::Error),
    ParseInt(std::num::ParseIntError),
    V1Protocol(v1::error::Error<'a>),
    VecToSlice32(Vec<u8>),
}

impl<'a> From<Vec<u8>> for TProxyBridgeError<'a> {
    fn from(e: Vec<u8>) -> Self {
        TProxyBridgeError::VecToSlice32(e)
    }
}

impl<'a> From<binary_sv2::Error> for TProxyBridgeError<'a> {
    fn from(e: binary_sv2::Error) -> Self {
        TProxyBridgeError::BinarySv2(e)
    }
}

impl<'a> From<std::num::ParseIntError> for TProxyBridgeError<'a> {
    fn from(e: std::num::ParseIntError) -> Self {
        TProxyBridgeError::ParseInt(e)
    }
}

impl<'a> From<async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>>
for TProxyBridgeError<'a>
{
    fn from(
        e: async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>,
    ) -> Self {
        TProxyBridgeError::ChannelSender(BridgeChannelSendError::SubmitSharesExtended(e))
    }
}

impl<'a> From<async_channel::RecvError> for TProxyBridgeError<'a> {
    fn from(e: async_channel::RecvError) -> Self {
        TProxyBridgeError::ChannelReceiver(e)
    }
}

impl<'a> From<tokio::sync::broadcast::error::SendError<Notify<'a>>> for TProxyBridgeError<'a> {
    fn from(e: tokio::sync::broadcast::error::SendError<Notify<'a>>) -> Self {
        TProxyBridgeError::ChannelSender(BridgeChannelSendError::Notify(e))
    }
}

impl<'a> From<roles_logic_sv2::errors::Error> for TProxyBridgeError<'a> {
    fn from(e: roles_logic_sv2::errors::Error) -> Self {
        TProxyBridgeError::RolesSv2Logic(e)
    }
}