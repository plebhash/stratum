use std::fmt;
use roles_logic_sv2::mining_sv2::{ExtendedExtranonce, NewExtendedMiningJob, SetCustomMiningJob};
use v1::server_to_client::Notify;

#[derive(Debug)]
pub enum UpstreamChannelSendError<'a> {
    SubmitSharesExtended(
        async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>,
    ),
    SetNewPrevHash(async_channel::SendError<roles_logic_sv2::mining_sv2::SetNewPrevHash<'a>>),
    NewExtendedMiningJob(async_channel::SendError<NewExtendedMiningJob<'a>>),
    Notify(tokio::sync::broadcast::error::SendError<Notify<'a>>),
    V1Message(async_channel::SendError<v1::Message>),
    General(String),
    Extranonce(async_channel::SendError<(ExtendedExtranonce, u32)>),
    SetCustomMiningJob(
        async_channel::SendError<roles_logic_sv2::mining_sv2::SetCustomMiningJob<'a>>,
    ),
    NewTemplate(
        async_channel::SendError<(
            roles_logic_sv2::template_distribution_sv2::SetNewPrevHash<'a>,
            Vec<u8>,
        )>,
    ),
}

pub type TProxyUpstreamResult<'a, T> = Result<T, TProxyUpstreamError<'a>>;

#[derive(Debug)]
pub enum TProxyUpstreamError<'a> {
    PoisonLock,
    Io(std::io::Error),
    CodecNoise(codec_sv2::noise_sv2::Error),
    RolesSv2Logic(roles_logic_sv2::errors::Error),
    BinarySv2(binary_sv2::Error),
    FramingSv2(framing_sv2::Error),
    ChannelSender(UpstreamChannelSendError<'a>),
    ChannelReceiver(async_channel::RecvError),
    InvalidExtranonce(String),
    SubprotocolMining(String),
}

impl<'a> fmt::Display for TProxyUpstreamError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TProxyUpstreamError::PoisonLock => write!(f, "Poisoned lock detected"),
            TProxyUpstreamError::Io(err) => write!(f, "IO error: {}", err),
            TProxyUpstreamError::CodecNoise(err) => write!(f, "Codec noise error: {:?}", err),
            TProxyUpstreamError::RolesSv2Logic(err) => write!(f, "Roles SV2 Logic error: {}", err),
            TProxyUpstreamError::BinarySv2(err) => write!(f, "Binary SV2 error: {:?}", err),
            TProxyUpstreamError::FramingSv2(err) => write!(f, "Framing SV2 error: `{:?}`", err),
            TProxyUpstreamError::ChannelSender(err) => write!(f, "Channel Sender error: {:?}", err),
            TProxyUpstreamError::ChannelReceiver(err) => write!(f, "Channel Receiver error: {:?}", err),
            TProxyUpstreamError::InvalidExtranonce(err) => write!(f, "Invalid Extranonce error: {:?}", err),
            TProxyUpstreamError::SubprotocolMining(err) => write!(f, "Subprotocol Mining error: {:?}", err)
        }
    }
}

impl<'a> From<std::io::Error> for TProxyUpstreamError<'a> {
    fn from(e: std::io::Error) -> Self {
        TProxyUpstreamError::Io(e)
    }
}

impl<'a> From<async_channel::RecvError> for TProxyUpstreamError<'a> {
    fn from(e: async_channel::RecvError) -> Self {
        TProxyUpstreamError::ChannelReceiver(e)
    }
}

impl<'a> From<codec_sv2::noise_sv2::Error> for TProxyUpstreamError<'a> {
    fn from(e: codec_sv2::noise_sv2::Error) -> Self {
        TProxyUpstreamError::CodecNoise(e)
    }
}

impl<'a> From<roles_logic_sv2::errors::Error> for TProxyUpstreamError<'a> {
    fn from(e: roles_logic_sv2::errors::Error) -> Self {
        TProxyUpstreamError::RolesSv2Logic(e)
    }
}

impl<'a> From<binary_sv2::Error> for TProxyUpstreamError<'a> {
    fn from(e: binary_sv2::Error) -> Self {
        TProxyUpstreamError::BinarySv2(e)
    }
}

impl<'a> From<framing_sv2::Error> for TProxyUpstreamError<'a> {
    fn from(e: framing_sv2::Error) -> Self {
        TProxyUpstreamError::FramingSv2(e)
    }
}

// *** CHANNEL SENDER ERRORS ***
impl<'a> From<async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>>
for TProxyUpstreamError<'a>
{
    fn from(
        e: async_channel::SendError<roles_logic_sv2::mining_sv2::SubmitSharesExtended<'a>>,
    ) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::SubmitSharesExtended(e))
    }
}

impl<'a> From<async_channel::SendError<roles_logic_sv2::mining_sv2::SetNewPrevHash<'a>>>
for TProxyUpstreamError<'a>
{
    fn from(e: async_channel::SendError<roles_logic_sv2::mining_sv2::SetNewPrevHash<'a>>) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::SetNewPrevHash(e))
    }
}

impl<'a> From<tokio::sync::broadcast::error::SendError<Notify<'a>>> for TProxyUpstreamError<'a> {
    fn from(e: tokio::sync::broadcast::error::SendError<Notify<'a>>) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::Notify(e))
    }
}

impl<'a> From<async_channel::SendError<v1::Message>> for TProxyUpstreamError<'a> {
    fn from(e: async_channel::SendError<v1::Message>) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::V1Message(e))
    }
}

impl<'a> From<async_channel::SendError<(ExtendedExtranonce, u32)>> for TProxyUpstreamError<'a> {
    fn from(e: async_channel::SendError<(ExtendedExtranonce, u32)>) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::Extranonce(e))
    }
}

impl<'a> From<async_channel::SendError<NewExtendedMiningJob<'a>>> for TProxyUpstreamError<'a> {
    fn from(e: async_channel::SendError<NewExtendedMiningJob<'a>>) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::NewExtendedMiningJob(e))
    }
}

impl<'a> From<async_channel::SendError<SetCustomMiningJob<'a>>> for TProxyUpstreamError<'a> {
    fn from(e: async_channel::SendError<SetCustomMiningJob<'a>>) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::SetCustomMiningJob(e))
    }
}

impl<'a>
From<
    async_channel::SendError<(
        roles_logic_sv2::template_distribution_sv2::SetNewPrevHash<'a>,
        Vec<u8>,
    )>,
> for TProxyUpstreamError<'a>
{
    fn from(
        e: async_channel::SendError<(
            roles_logic_sv2::template_distribution_sv2::SetNewPrevHash<'a>,
            Vec<u8>,
        )>,
    ) -> Self {
        TProxyUpstreamError::ChannelSender(UpstreamChannelSendError::NewTemplate(e))
    }
}


