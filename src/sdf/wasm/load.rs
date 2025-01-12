use std::future::Future;

use anyhow::anyhow;
use ehttp::Request;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::metadata::short_version_info_is_ours;

use crate::sdf::SDFSurface;
use crate::sdf::wasm::native;
use crate::sdf::demo::SDFDemo;

/// See [`load_sdf_wasm`] for more information.
pub async fn load_sdf_wasm_send_sync(wasm_bytes: &[u8]) -> anyhow::Result<Box<dyn SDFSurface + Send + Sync>> {
    native::load_sdf_wasm_send_sync(wasm_bytes).await
}

/// Loads the given bytes as a WebAssembly module that is then queried to satisfy the SDF trait.
///
/// It uses the default WebAssembly interpreter for the platform.
#[allow(dead_code)] // Not used in the current implementation because Send + Sync is needed for the WebAssembly engine.
pub async fn load_sdf_wasm(wasm_bytes: &[u8]) -> anyhow::Result<Box<dyn SDFSurface>> {
    native::load_sdf_wasm(wasm_bytes).await
}

/// Abstraction over [`load_sdf_wasm`] that allows it to load from a file path or HTTP server URL,
/// providing automatic updates for the latter (if the server supports it).
pub fn load_sdf_from_path_or_url(sender_of_updates: Sender<Receiver<Box<dyn SDFSurface + Send + Sync>>>, watch_url: String) {
    ehttp::fetch(Request::get(watch_url.clone()), move |data| {
        handle_sdf_data_response(data, watch_url, sender_of_updates)
    });
}

/// Native: creates a new thread with a new async runtime that blocks on the given task.
/// Web: spawns the asynchronous task. Note that it SHOULD NOT BLOCK as it actually runs concurrently
/// in the main thread.
///
/// The new_runtime parameter is required in native to specify if the current thread has a runtime or not.
/// Both cases will continue the execution even if the task is not completed.
pub fn spawn_async(fut: impl Future<Output=()> + Send + 'static, new_runtime: bool) {
    if !new_runtime {
        // After creating a new thread on native, it needs a new tokio runtime to block on a future!
        tokio::spawn(fut);
    } else {
        // To follow the convention of not blocking, we create a new thread for the runtime and return immediately.
        std::thread::spawn(move || {
            #[cfg(not(target_arch = "wasm32"))]
            tokio::runtime::Builder::new_multi_thread().build().unwrap().block_on(fut);
            #[cfg(target_arch = "wasm32")]
            tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(fut);
        });
    }
}

/// This is a helper function to load a SDF from a WebAssembly binary. It initially tries to load the
/// HTTP response as a WebAssembly binary, but falls back to loading it as a local file if that fails.
fn handle_sdf_data_response(data: ehttp::Result<ehttp::Response>, watch_url_closure: String,
                            sender_of_updates: Sender<Receiver<Box<dyn SDFSurface + Send + Sync>>>) {
    // First, try to request the file as an URL on any platform (with some fallbacks).
    let fut = async move {
        let (sender_single_update, receiver_single_update) = mpsc::channel(1);
        if sender_of_updates.send(receiver_single_update).await.is_err() {
            tracing::warn!("The listener ignored our update notification, won't send more notifications");
            return; // Stop recursion here
        }
        let res = match data {
            Ok(resp) => {
                // If the server properly supports the ?watch query parameter, we can start checking for changes.
                // tracing::info!("HTTP headers: {:?}", resp.headers);
                let supports_watching_pre = {
                    #[cfg(not(target_arch = "wasm32"))]
                    { false }
                    // Web seems to have trouble recording the previous response headers, so try even harder
                    #[cfg(target_arch = "wasm32")]
                    { resp.headers.get("expires").map(|v| v == "123456").unwrap_or(false) }
                };
                let supports_watching = supports_watching_pre || // NOTE: This is a hacky way to detect whether the server supports the ?watch query parameter.
                    resp.headers.get("x-watch-supported").map(|_v| true).unwrap_or(false) ||
                    resp.headers.get("server").map(short_version_info_is_ours).unwrap_or(false);
                if supports_watching {
                    tracing::info!("Server supports watching for file changes, enabling continuous updates.");
                    // Queue a ?watch request to the server, which will wait for source updates, recompile and return the new WASM file!
                    let watch_url_closure_clone = watch_url_closure.clone();
                    ehttp::fetch(Request::get(watch_url_closure.clone() + "?watch"), move |data| {
                        handle_sdf_data_response(data, watch_url_closure_clone, sender_of_updates)
                    });
                } else {
                    // Otherwise, give up on continuous updates by dropping the sender_of_updates!
                    tracing::warn!("HTTP Server does not support watching for file changes, disabling continuous updates.");
                    drop(sender_of_updates); // This is not needed, but states what we want
                }
                // TODO: Avoid this blocking code...
                load_sdf_wasm_send_sync(resp.bytes.as_slice()).await
            }
            Err(err_str) => Err(anyhow::anyhow!(err_str)),
        };
        let channel_send_err = match res {
            Ok(sdf) => { // If successful, load it now
                sender_single_update.send(sdf).await.map_err(|_| anyhow!("can't send SDF update"))
            }
            Err(err) => { // If not, try to load it as a local file on native platforms.
                #[cfg(target_arch = "wasm32")]
                {
                    tracing::error!("Failed to load SDF from URL: {:?}", err);
                    sender_single_update.try_send(unsafe {
                        // FIXME: Extremely unsafe code (forcing SDFDemo Send+Sync), but only used for this error path
                        std::mem::transmute::<Box<dyn SDFSurface>, Box<dyn SDFSurface + Send + Sync>>(Box::new(SDFDemo::default()) as Box<dyn SDFSurface>)
                    }).map_err(|_| anyhow!("can't send SDF update"))
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // TODO: Avoid this blocking code...
                    let res = match std::fs::read(watch_url_closure) {
                        Ok(bytes) => {
                            // TODO: Avoid this blocking code...
                            load_sdf_wasm_send_sync(bytes.as_slice()).await
                        }
                        Err(err) => Err(anyhow::Error::from(err)),
                    };
                    match res {
                        Ok(sdf) => {
                            sender_single_update.send(sdf).await.map_err(|_| anyhow!("can't send SDF update"))
                        }
                        Err(err2) => {
                            tracing::error!("Failed to load SDF from URL ({:?}) or file ({:?})", err, err2);
                            sender_single_update.try_send(unsafe {
                                // FIXME: Extremely unsafe code (forcing SDFDemo Send+Sync), but only used for this error path
                                std::mem::transmute::<Box<dyn SDFSurface>, Box<dyn SDFSurface + Send + Sync>>(Box::<SDFDemo>::default() as Box<dyn SDFSurface>)
                            }).map_err(|_| anyhow!("can't send SDF update"))
                        }
                    }
                }
            }
        };
        if channel_send_err.is_err() {
            tracing::warn!("The listener ignored our update notification (2)");
        }
    };
    spawn_async(fut, true)
}
