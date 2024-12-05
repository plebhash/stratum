#![deny(missing_docs)]
use roles_logic_sv2::parsers::Mining;

use super::error::PoolError;

/// Each sending side of the status channel
/// should be wrapped with this enum to allow
/// the main thread to know which component sent the message
#[derive(Debug)]
pub enum Sender {
    Downstream(async_channel::Sender<Status>),
    DownstreamListener(async_channel::Sender<Status>),
    Upstream(async_channel::Sender<Status>),
}

impl Sender {
    /// used to clone the sending side of the status channel used by the TCP Listener
    /// into individual Sender's for each Downstream instance
    pub fn listener_to_connection(&self) -> Self {
        match self {
            // should only be used to clone the DownstreamListener(Sender) into Downstream(Sender)s
            Self::DownstreamListener(inner) => Self::Downstream(inner.clone()),
            _ => unreachable!(),
        }
    }

    pub async fn send(&self, status: Status) -> Result<(), async_channel::SendError<Status>> {
        match self {
            Self::Downstream(inner) => inner.send(status).await,
            Self::DownstreamListener(inner) => inner.send(status).await,
            Self::Upstream(inner) => inner.send(status).await,
        }
    }
}

impl Clone for Sender {
    fn clone(&self) -> Self {
        match self {
            Self::Downstream(inner) => Self::Downstream(inner.clone()),
            Self::DownstreamListener(inner) => Self::DownstreamListener(inner.clone()),
            Self::Upstream(inner) => Self::Upstream(inner.clone()),
        }
    }
}

#[derive(Debug)]
pub enum State {
    DownstreamShutdown(PoolError),
    TemplateProviderShutdown(PoolError),
    DownstreamInstanceDropped(u32),
    Healthy(String),
}

/// message to be sent to the status loop on the main thread
#[derive(Debug)]
pub struct Status {
    pub state: State,
}

/// this function is used to discern which componnent experienced the event.
/// With this knowledge we can wrap the status message with information (`State` variants) so
/// the main status loop can decide what should happen
async fn send_status(
    sender: &Sender,
    e: PoolError,
    outcome: error_handling::ErrorBranch,
) -> error_handling::ErrorBranch {
    match sender {
        Sender::Downstream(tx) => match e {
            PoolError::Sv2ProtocolError((id, Mining::OpenMiningChannelError(_))) => {
                tx.send(Status {
                    state: State::DownstreamInstanceDropped(id),
                })
                .await
                .unwrap_or(());
            }
            _ => {
                let string_err = e.to_string();
                tx.send(Status {
                    state: State::Healthy(string_err),
                })
                .await
                .unwrap_or(());
            }
        },
        Sender::DownstreamListener(tx) => match e {
            PoolError::RolesLogic(roles_logic_sv2::Error::NoDownstreamsConnected) => {
                tx.send(Status {
                    state: State::Healthy("No Downstreams Connected".to_string()),
                })
                .await
                .unwrap_or(());
            }
            _ => {
                tx.send(Status {
                    state: State::DownstreamShutdown(e),
                })
                .await
                .unwrap_or(());
            }
        },
        Sender::Upstream(tx) => {
            tx.send(Status {
                state: State::TemplateProviderShutdown(e),
            })
            .await
            .unwrap_or(());
        }
    }
    outcome
}

// this is called by `error_handling::handle_result!`
// todo: as described in issue #777, we should replace every generic *(_) with specific errors and
// cover every possible combination
pub async fn handle_error(sender: &Sender, e: PoolError) -> error_handling::ErrorBranch {
    tracing::debug!("Error: {:?}", &e);
    match e {
        PoolError::Io(_) => send_status(sender, e, error_handling::ErrorBranch::Break).await,
        PoolError::ChannelSend(_) => {
            //This should be a continue because if we fail to send to 1 downstream we should
            // continue processing the other downstreams in the loop we are in.
            // Otherwise if a downstream fails to send to then subsequent downstreams in
            // the map won't get send called on them
            send_status(sender, e, error_handling::ErrorBranch::Continue).await
        }
        PoolError::ChannelRecv(_) => {
            send_status(sender, e, error_handling::ErrorBranch::Break).await
        }
        PoolError::BinarySv2(_) => send_status(sender, e, error_handling::ErrorBranch::Break).await,
        PoolError::Codec(_) => send_status(sender, e, error_handling::ErrorBranch::Break).await,
        PoolError::Noise(_) => send_status(sender, e, error_handling::ErrorBranch::Continue).await,
        PoolError::RolesLogic(roles_logic_sv2::Error::NoDownstreamsConnected) => {
            send_status(sender, e, error_handling::ErrorBranch::Continue).await
        }
        PoolError::RolesLogic(_) => {
            send_status(sender, e, error_handling::ErrorBranch::Break).await
        }
        PoolError::Custom(_) => send_status(sender, e, error_handling::ErrorBranch::Break).await,
        PoolError::Framing(_) => send_status(sender, e, error_handling::ErrorBranch::Break).await,
        PoolError::PoisonLock(_) => {
            send_status(sender, e, error_handling::ErrorBranch::Break).await
        }
        PoolError::ComponentShutdown(_) => {
            send_status(sender, e, error_handling::ErrorBranch::Break).await
        }
        PoolError::Sv2ProtocolError(_) => {
            send_status(sender, e, error_handling::ErrorBranch::Break).await
        }
    }
}
