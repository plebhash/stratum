#![allow(special_module_name)]
use async_channel::{bounded, unbounded};
use tokio::select;
use key_utils::{Secp256k1PublicKey, Secp256k1SecretKey};
//use error::OutputScriptError;
use roles_logic_sv2::{
    errors::Error, parsers::PoolMessages, utils::CoinbaseOutput as CoinbaseOutput_,
};

use tracing::{error, info, warn};
// use pool_sv2::template_receiver;

mod lib;

// use lib::{mining_pool::Pool, template_receiver::TemplateRx};

mod args {
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct Args {
        pub config_path: PathBuf,
    }

    enum ArgsState {
        Next,
        ExpectPath,
        Done,
    }

    enum ArgsResult {
        Config(PathBuf),
        None,
        Help(String),
    }

    impl Args {
        const DEFAULT_CONFIG_PATH: &'static str = "pool-config.toml";

        pub fn from_args() -> Result<Self, String> {
            let cli_args = std::env::args();

            let config_path = cli_args
                .scan(ArgsState::Next, |state, item| {
                    match std::mem::replace(state, ArgsState::Done) {
                        ArgsState::Next => match item.as_str() {
                            "-c" | "--config" => {
                                *state = ArgsState::ExpectPath;
                                Some(ArgsResult::None)
                            }
                            "-h" | "--help" => Some(ArgsResult::Help(format!(
                                "Usage: -h/--help, -c/--config <path|default {}>",
                                Self::DEFAULT_CONFIG_PATH
                            ))),
                            _ => {
                                *state = ArgsState::Next;

                                Some(ArgsResult::None)
                            }
                        },
                        ArgsState::ExpectPath => Some(ArgsResult::Config(PathBuf::from(item))),
                        ArgsState::Done => None,
                    }
                })
                .last();
            let config_path = match config_path {
                Some(ArgsResult::Config(p)) => p,
                Some(ArgsResult::Help(h)) => return Err(h),
                _ => PathBuf::from(Self::DEFAULT_CONFIG_PATH),
            };
            Ok(Self { config_path })
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = match args::Args::from_args() {
        Ok(cfg) => cfg,
        Err(help) => {
            error!("{}", help);
            return;
        }
    };

    // Load config
    let config: pool_sv2::Configuration = match std::fs::read_to_string(&args.config_path) {
        Ok(c) => match toml::from_str(&c) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to parse config: {}", e);
                return;
            }
        },
        Err(e) => {
            error!("Failed to read config: {}", e);
            return;
        }
    };

    let (status_tx, status_rx) = unbounded();
    let (s_new_t, r_new_t) = bounded(10);
    let (s_prev_hash, r_prev_hash) = bounded(10);
    let (s_solution, r_solution) = bounded(10);
    let (s_message_recv_signal, r_message_recv_signal) = bounded(10);
    info!("Pool INITIALIZING with config: {:?}", &args.config_path);
    let coinbase_output_result = pool_sv2::get_coinbase_output(&config);
    let coinbase_output_len = match coinbase_output_result {
        Ok(coinbase_output) => coinbase_output.len() as u32,
        Err(err) => {
            error!("Failed to get coinbase output: {:?}", err);
            return;
        }
    };
    let authority_public_key = config.authority_public_key.clone();
    let template_rx_res = lib::template_receiver::TemplateRx::connect(
        config.tp_address.parse().unwrap(),
        s_new_t,
        s_prev_hash,
        r_solution,
        r_message_recv_signal,
        status::Sender::Upstream(status_tx.clone()),
        coinbase_output_len,
        authority_public_key,
    )
    .await;

    if let Err(e) = template_rx_res {
        error!("Could not connect to Template Provider: {}", e);
        return;
    }

    let pool = Pool::start(
        config.clone(),
        r_new_t,
        r_prev_hash,
        s_solution,
        s_message_recv_signal,
        status::Sender::DownstreamListener(status_tx),
    );

    // Start the error handling loop
    // See `./status.rs` and `utils/error_handling` for information on how this operates
    loop {
        let task_status = select! {
            task_status = status_rx.recv() => task_status,
            interrupt_signal = tokio::signal::ctrl_c() => {
                match interrupt_signal {
                    Ok(()) => {
                        info!("Interrupt received");
                    },
                    Err(err) => {
                        error!("Unable to listen for interrupt signal: {}", err);
                        // we also shut down in case of error
                    },
                }
                break;
            }
        };
        let task_status: crate::status::Status = task_status.unwrap();

        match task_status.state {
            // Should only be sent by the downstream listener
            status::State::DownstreamShutdown(err) => {
                error!(
                    "SHUTDOWN from Downstream: {}\nTry to restart the downstream listener",
                    err
                );
                break;
            }
            status::State::TemplateProviderShutdown(err) => {
                error!("SHUTDOWN from Upstream: {}\nTry to reconnecting or connecting to a new upstream", err);
                break;
            }
            status::State::Healthy(msg) => {
                info!("HEALTHY message: {}", msg);
            }
            status::State::DownstreamInstanceDropped(downstream_id) => {
                warn!("Dropping downstream instance {} from pool", downstream_id);
                if pool
                    .safe_lock(|p| p.remove_downstream(downstream_id))
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}
