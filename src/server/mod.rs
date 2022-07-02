use std::ffi::OsString;
use std::net::IpAddr;

use salvo::http::response::Body;
use salvo::prelude::*;
use salvo::routing::{Filter, PathState};

#[derive(clap::Parser, Debug, Clone, PartialEq, Eq)]
pub struct CliServer {
    /// The path of the files that will be served. Usually just one *.wasm file.
    #[clap(short, long)]
    pub serve_paths: Vec<String>,
    /// The path of the files to watch for changes. Leave empty to avoid watching files and notifying the viewer.
    #[clap(short, long)]
    pub watch_paths: Vec<String>,
    /// An optional command to run when a file changes. Useful to automate builds watching source
    /// code changes instead of directly watching the build results.
    #[clap(short, long)]
    pub command: Option<String>,
    /// The host to listen on.
    #[clap(short = 'l', long, default_value = "127.0.0.1")]
    pub host: IpAddr,
    /// The port to listen on.
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
}

impl Default for CliServer {
    fn default() -> Self {
        use clap::Parser;
        Self::parse_from::<_, OsString>([])
    }
}

impl CliServer {
    /// Runs the server forever (requires async context).
    pub async fn run(self) {
        tracing::info!("Starting server with configuration {:?}", self);

        #[derive(Debug)]
        struct AnyFilter;

        impl Filter for AnyFilter {
            fn filter(&self, _req: &mut Request, path: &mut PathState) -> bool {
                *path = PathState::new(""); // HACK: Forces any filter to match.
                true
            }
        }

        // Files that will be served
        struct FileServerHandler {
            cfg: CliServer,
        }
        #[async_trait]
        impl Handler for FileServerHandler {
            async fn handle(&self, req: &mut Request, _depot: &mut Depot, res: &mut Response, _ctrl: &mut FlowCtrl) {
                // Parse input request
                let file_path = req.uri().path().strip_prefix('/').unwrap_or_default();
                let watch_for_changes = req.query::<String>("watch").map(|_| true /* any value is true */).unwrap_or(false);
                tracing::info!("Handling request of file {} with watch/compile {}", file_path, watch_for_changes);
                // Validate file path
                if !self.cfg.serve_paths.iter().any(|p| p == file_path) {
                    tracing::error!("Received request to file that is not public {}", file_path);
                    res.set_status_error(StatusError::not_found());
                    return;
                }
                // Watch (& compile) file if requested
                if watch_for_changes {
                    todo!("Watch file");
                }
                // Serve file
                match tokio::fs::read(file_path).await { // TODO: Streaming?
                    Ok(file_bytes) => {
                        res.set_body(Body::from(salvo::hyper::Body::from(file_bytes)));
                    }
                    Err(err) => {
                        tracing::error!("Failed to open file {}: {}", file_path, err);
                        res.set_status_error(StatusError::not_found());
                    }
                };
            }
        }
        let router = Router::new().filter(AnyFilter {}).get(FileServerHandler { cfg: self.clone() });

        // Await forever serving requests
        Server::new(TcpListener::bind((self.host, self.port)))
            .serve(router).await;
    }
}
