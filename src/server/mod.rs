use std::ffi::OsString;
use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

use httpdate::fmt_http_date;
use lru::LruCache;
use notify_debouncer_full::notify::{event::AccessKind, EventKind, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use salvo::conn::Acceptor;
use salvo::http::header::HeaderName;
use salvo::http::response::ResBody;
use salvo::http::{HeaderMap, HeaderValue};
use salvo::prelude::*;
use salvo::routing::{Filter, PathState};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::Mutex;

use crate::metadata::short_version_info;

#[derive(clap::Parser, Debug, Clone, PartialEq, Eq)]
pub struct CliServer {
    /// The path of the files that will be served. Usually just one *.wasm file. Only exact path
    /// matches will be served.
    #[clap(short, long)]
    pub serve_paths: Vec<String>,
    /// The path of the files to watch for changes. Leave empty to avoid watching files and notifying the viewer.
    #[clap(short, long)]
    pub watch_paths: Vec<String>,
    /// Wait for this amount of nanoseconds after each modification detected to merge modifications
    /// (for example, when saving multiple files at the same time).
    /// This is useful to avoid too many notifications, but adds a delay in the detection of changes.
    #[clap(short = 't', long, parse(try_from_str = parse_duration_ns), default_value = "12345678")]
    pub watch_merge_ns: Duration,
    /// An optional command to run when a file changes. Useful to automate builds watching source
    /// code changes instead of directly watching the build results.
    #[clap(short, long)]
    pub build_command: Vec<String>,
    /// The host to listen on.
    #[clap(short = 'l', long, default_value = "127.0.0.1")]
    pub host: IpAddr,
    /// The port to listen on.
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
}

fn parse_duration_ns(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(Duration::from_nanos(seconds))
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

        #[async_trait]
        impl Filter for AnyFilter {
            async fn filter(&self, _req: &mut Request, path: &mut PathState) -> bool {
                *path = PathState::new(""); // HACK: Forces any filter to match.
                true
            }
        }

        /// Main handler to serve the files.
        struct FileServerHandler {
            /// The main configuration
            cfg: CliServer,
            /// The sender to subscribe for file changes.
            sender: Sender<u64>,
            /// Event sequential ID receivers for the watched files.
            /// This keeps track of whether each client (up to a limit) has watch notifications
            /// pending, so that if watch is requested again it can immediately return, solving
            /// races.
            remote_events: Mutex<LruCache<String, Receiver<u64>>>,
            /// The last build used the files at least as recent as this event ID.
            /// It is behind a lock to allow interior mutability, and it is also useful as a
            /// build lock to avoid spawning multiple build processes at the same time...
            last_build_event: Mutex<u64>,
        }

        /// Implementation of the main handler
        #[async_trait]
        impl Handler for FileServerHandler {
            async fn handle(&self, req: &mut Request, _depot: &mut Depot, res: &mut Response, _ctrl: &mut FlowCtrl) {
                // Parse input request
                let file_path = req.uri().path().strip_prefix('/').unwrap_or_default();
                let watch_for_changes = req.query::<String>("watch").map(|_| true /* any value is true */).unwrap_or(false);
                tracing::info!(file_path=file_path, watch_for_changes=watch_for_changes, "Handling request of file");

                // Response headers
                let mut headers = HeaderMap::new();
                headers.insert(
                    salvo::http::header::CACHE_CONTROL,
                    HeaderValue::from_static("no-cache"),
                );
                headers.insert(
                    salvo::http::header::EXPIRES,
                    HeaderValue::from_static("123456"),
                );
                headers.insert(
                    salvo::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    HeaderValue::from_static("*"),
                );
                headers.insert(
                    salvo::http::header::ACCESS_CONTROL_ALLOW_HEADERS,
                    HeaderValue::from_static("*"),
                );
                headers.insert(
                    salvo::http::header::ACCESS_CONTROL_ALLOW_METHODS,
                    HeaderValue::from_static("GET"),
                );
                headers.insert(
                    salvo::http::header::SERVER,
                    HeaderValue::from_str(&short_version_info()).unwrap(),
                );
                headers.insert(
                    salvo::http::header::SET_COOKIE,
                    HeaderValue::from_str(&("server=".to_string() + &short_version_info())).unwrap(),
                );

                // Validate file path
                if !self.cfg.serve_paths.iter().any(|p| p == file_path) {
                    tracing::error!("Received request to file that is not public {}", file_path);
                    StatusError::not_found().render(res);
                    res.set_headers(headers);
                    return;
                }

                // Identify caller to provide it's own updates
                let remote_id = match req.remote_addr() {
                    // The same remote will open connections on different ports, so we need to
                    // use only the remote IP to identify the connection. However, this may cause
                    // collisions if the same IP is used on multiple machines (NAT).
                    salvo::core::conn::addr::SocketAddr::IPv4(addr) => Some(addr.ip().to_string()),
                    salvo::core::conn::addr::SocketAddr::IPv6(addr) => Some(addr.ip().to_string()),
                    _ => None,
                }.unwrap_or_else(|| "unknown".to_string());
                let mut build_event = 0;

                // Watch (& compile) file if requested
                if watch_for_changes {

                    // Event mutex sync
                    {
                        // Check for updates from the file watcher thread for this specific client
                        tracing::info!(requested_file=file_path, remote_id=remote_id, "Locking the event receiver"); // TODO: Reuse builds if possible
                        let mut remote_events_table = self.remote_events.lock().await; // Will be dropped after build finishes.
                        remote_events_table.get_or_insert(remote_id.clone(), || self.sender.subscribe());
                        let events = remote_events_table.get_mut(&remote_id).unwrap(); // Safe because we just inserted it.

                        // Wait for the first event.
                        // TODO: Release mutex while this user waits for their event.
                        tracing::info!(requested_file=file_path, remote_id=remote_id, "Waiting for changes");
                        // Errors (event capacity overflow) force a rebuild even if the file is not changed.
                        build_event = events.recv().await.unwrap_or(u64::MAX);
                        loop { // Aggregate the following events until the timeout is reached.
                            match tokio::time::timeout(self.cfg.watch_merge_ns, events.recv()).await {
                                Ok(Ok(event)) => build_event = event,
                                Ok(Err(RecvError::Lagged(by))) => {
                                    tracing::warn!(requested_file=file_path, remote_id=remote_id, "Receiver lagged behind (by {} events), try increasing the event capacity!", by);
                                    *events = self.sender.subscribe(); // Resubscribe to the new channel.
                                }
                                Ok(Err(_)) => panic!("Unexpected error from the event receiver"),
                                Err(_) => break, // Timeout, so stop merging events
                            }
                        }
                    }

                    // "Compile" if needed and configured
                    if !self.cfg.build_command.is_empty() {
                        // Build mutex sync
                        let mut last_build_event_mut = self.last_build_event.lock().await;
                        if build_event <= *last_build_event_mut {
                            tracing::info!(requested_file=file_path, remote_id=remote_id, build_event=build_event, last_build_event_mut=*last_build_event_mut, "Build command skipped");
                        } else {
                            self.perform_build(res, file_path, &remote_id, build_event).await;
                            if build_event != u64::MAX {
                                *last_build_event_mut = build_event;
                            }
                        }
                    }
                }

                // "Compile" if configured (no way to know if needed)
                if build_event == 0 && !self.cfg.build_command.is_empty() {
                    self.perform_build(res, file_path, &remote_id, build_event).await;
                }

                // Extract file metadata
                let metadata_fut = tokio::fs::metadata(file_path).await;
                // Serve file
                match metadata_fut.and_then(|m| std::fs::read(file_path).map(|r| (m, r))) { // TODO: Streaming?
                    Ok((metadata, file_bytes)) => {
                        headers.insert(
                            HeaderName::from_static("x-watch-supported"),
                            HeaderValue::from_static("true"),
                        );
                        headers.insert(
                            salvo::http::header::CONTENT_TYPE,
                            HeaderValue::from_static("application/wasm"),
                        );
                        headers.insert(
                            salvo::http::header::CONTENT_LENGTH,
                            HeaderValue::from_str(&file_bytes.len().to_string())
                                .unwrap_or_else(|_| HeaderValue::from_str("error").unwrap()),
                        );
                        headers.insert(
                            salvo::http::header::LAST_MODIFIED,
                            HeaderValue::from_str(&fmt_http_date(
                                metadata.modified().unwrap_or_else(|_| SystemTime::now())))
                                .unwrap_or_else(|_| HeaderValue::from_str("error").unwrap()),
                        );
                        res.body(ResBody::from(file_bytes));
                    }
                    Err(err) => {
                        tracing::error!("Failed to read file {}: {}", file_path, err);
                        StatusError::not_found().render(res);
                    }
                };
                res.set_headers(headers);
            }
        }

        impl FileServerHandler {
            async fn perform_build(&self, res: &mut Response, file_path: &str, remote_id: &String, build_event: u64) {
                let mut cmd = tokio::process::Command::new(&self.cfg.build_command[0]);
                cmd.args(self.cfg.build_command.iter().skip(1).map(|el|
                    el.strip_prefix('\\').unwrap_or(el)))
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());
                tracing::info!(requested_file=file_path, remote_id=remote_id, cmd=format!("{cmd:?}"), "Starting build");
                let ret = cmd.spawn().expect("Bad build command!").wait().await;
                match ret {
                    Ok(status) => {
                        if !status.success() {
                            tracing::error!(requested_file=file_path, remote_id=remote_id, build_event=build_event, "Build command failed");
                            StatusError::internal_server_error().render(res);
                            return;
                        }
                    }
                    Err(e) => {
                        tracing::error!(requested_file=file_path, remote_id=remote_id, build_event=build_event, "Build command failed: {}", e);
                        StatusError::internal_server_error().render(res);
                        return;
                    }
                }
                tracing::info!(requested_file=file_path, remote_id=remote_id, build_event=build_event, "Build completed successfully");
            }
        }

        // Start the change watcher
        let (modified_sender, _modified_receiver) = channel(1);
        let modified_sender_clone = modified_sender.clone();
        let watch_paths = self.watch_paths.clone();
        thread::spawn(move || {
            // Closure to handle errors easily.
            let run_thread = move || -> anyhow::Result<()> {
                let (tx, rx) = std::sync::mpsc::channel();

                // Select recommended watcher for debouncer.
                // Using a callback here, could also be a channel.
                let mut debouncer = new_debouncer(self.watch_merge_ns, None, move |res: DebounceEventResult| {
                    match res {
                        Ok(events) => {
                            events.iter().for_each(|e| println!("Event {:?} for {:?}", e.kind, e.paths));
                            if !events.is_empty() {
                                tx.send(events).unwrap();
                            }
                        }
                        Err(e) => tracing::error!("Error {:?}", e),
                    }
                })?;

                // Watch all files in the watch_paths (recursively if they are directories).
                for path in &watch_paths {
                    tracing::info!(path=path, "Recursively watching path for changes");
                    debouncer.watch(Path::new(path), RecursiveMode::Recursive)?;
                }

                let mut cur_event = 1u64;
                for x in rx {
                    if x.iter().all(|event| 
                        matches!(event.kind, EventKind::Access(AccessKind::Open(_)))) {
                        continue;
                    }
                    let notified = modified_sender_clone.send(cur_event)? - 1 /* initial receiver always available */;
                    tracing::info!(cur_event=cur_event, "Notifying of file update ({:?}) to {} receivers", x, notified);
                    cur_event += 1;
                }

                Err(anyhow::anyhow!("File watcher closed the events channel unexpectedly"))
            };

            match run_thread() {
                Ok(_) => (),
                Err(err) => {
                    tracing::error!("File watcher thread failed: {}. Won't receive any more watch updates!", err);
                }
            }
        });

        // Create the main router pointing to the main handler
        let router = Router::new().filter(AnyFilter {}).get(FileServerHandler {
            cfg: self.clone(),
            sender: modified_sender,
            remote_events: Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap())), // Up to N clients (without races that may skip events)
            last_build_event: Mutex::new(0),
        });

        // Await forever serving requests
        // TODO: Graceful shutdown
        let listener = TcpListener::new((self.host, self.port)).bind().await;
        tracing::info!(addr=listener.holdings()[0].to_string(), paths=format!("{:?}", self.serve_paths), "Listening for requests");
        Server::new(listener).serve(router).await;
    }
}
