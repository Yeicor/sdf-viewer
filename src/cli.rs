// TODO: Clap
// TODO: Web: read arguments from GET parameters

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

impl Cli {
    /// Parses from the command line arguments on native and from GET parameters on web.
    pub fn parse_args() -> Self {
        let args = {
            #[cfg(target_arch = "wasm32")]
            {
                let mut x = vec![env!("CARGO_PKG_NAME").to_string()];
                let location_string: String = web_sys::window().unwrap().location().to_string().into();
                let url = reqwest::Url::parse(location_string.as_str()).unwrap();
                for (k, v) in url.query_pairs() {
                    x.push(k.to_string());
                    if v.len() > 0 {
                        x.push(v.to_string());
                    }
                }
                if x.len() == 1 {
                    tracing::warn!("No arguments (GET parameters, try <url>?-h), setting defaults");
                    x.push("app".to_string());
                    x.push("demo".to_string());
                }
                x
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
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
