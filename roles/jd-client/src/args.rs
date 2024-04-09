use std::path::PathBuf;
use tracing::error;
use crate::lib::error::{JdcError, JdcResult};
use crate::lib::jdc_config::JdcConfig;

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
    const DEFAULT_CONFIG_PATH: &'static str = "jdc-config.toml";
    const HELP_MSG: &'static str = "Usage: -h/--help, -c/--config <path|default jdc-config.toml>";

    pub fn from_args() -> Result<Self, String> {
        let cli_args = std::env::args();

        if cli_args.len() == 1 {
            println!("Using default config path: {}", Self::DEFAULT_CONFIG_PATH);
            println!("{}\n", Self::HELP_MSG);
        }

        let config_path = cli_args
            .scan(ArgsState::Next, |state, item| {
                match std::mem::replace(state, ArgsState::Done) {
                    ArgsState::Next => match item.as_str() {
                        "-c" | "--config" => {
                            *state = ArgsState::ExpectPath;
                            Some(ArgsResult::None)
                        }
                        "-h" | "--help" => Some(ArgsResult::Help(Self::HELP_MSG.to_string())),
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

/// Process CLI args, if any.
#[allow(clippy::result_large_err)]
pub fn process_cli_args<'a>() -> JdcResult<'a, JdcConfig> {
    let args = match Args::from_args() {
        Ok(cfg) => cfg,
        Err(help) => {
            error!("{}", help);
            return Err(JdcError::BadCliArgs);
        }
    };
    let config_file = std::fs::read_to_string(args.config_path)?;
    Ok(toml::from_str::<JdcConfig>(&config_file)?)
}
