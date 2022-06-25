// TODO: Clap
// TODO: Web: read arguments from GET parameters

use std::collections::HashMap;

use clap::once_cell::sync::OnceCell;
use clap::Parser;
use tracing::error;

use crate::app::cli::{CliApp, CliAppWatchFile};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Start the main application that displays SDFs
    App(CliApp),
    /// Run the server that watches the filesystem and provides the updated SDF to the app
    Server(CliAppWatchFile),
}

/// This holds the environment, as it is unsupported but abstracted on web.
static ENV: OnceCell<HashMap<String, String>> = OnceCell::new();

/// Retrieves an "environment variable". It automatically adds the prefix of the crate name to the key.
pub fn env_get(key: impl AsRef<str>) -> Option<String> {
    let key = format!("{}_{}", env!("CARGO_PKG_NAME"), key.as_ref());
    ENV.get().and_then(|env| env.get(key.as_str()).cloned())
}

impl Cli {
    /// Parses from the command line arguments on native and from GET parameters on web.
    pub fn parse_args() -> Self {
        let args = {
            #[cfg(target_arch = "wasm32")]
            {
                let mut args = vec![env!("CARGO_PKG_NAME").to_string()];
                let location_string: String = web_sys::window().unwrap().location().to_string().into();
                let url = reqwest::Url::parse(location_string.as_str()).unwrap();
                let mut env_vars = HashMap::new();
                args.extend(url.query_pairs().into_iter().filter_map(|(key, value)| {
                    if let Some(env_key) = key.strip_prefix("env") {
                        // Add the crate name as prefix automatically
                        let mut env_key = env_key.to_string();
                        env_key.insert_str(0, &*format!("{}_", env!("CARGO_PKG_NAME")));
                        env_vars.insert(env_key, value.to_string());
                        None
                    } else if let Some(cli_key) = key.strip_prefix("cli") {
                        Some([cli_key.to_string(), value.to_string()]) // Filtered later if value is empty
                    } else {
                        None
                    }
                }).flatten().filter(|el| !el.is_empty()));
                ENV.set(env_vars).unwrap();
                if args.len() == 1 {
                    // No arguments, show the demo mode.
                    tracing::warn!("No arguments (GET params prefixed with \"cli\", try <url>?cli-h), setting defaults");
                    tracing::info!("You can also set the environment variables with the prefix \"env\"");
                    args.push("app".to_string());
                    args.push("demo".to_string());
                }
                args
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                ENV.set(std::env::vars().collect::<HashMap<_, _>>()).unwrap();
                std::env::args().into_iter().collect::<Vec<_>>()
            }
        };
        let slf: clap::Result<Self> = Self::try_parse_from(args.iter());
        slf.unwrap_or_else(|e| {
            // Use tracing to avoid default crash on web
            error!("Error parsing arguments: {}", e);
            std::process::exit(0);
        })
    }
}
