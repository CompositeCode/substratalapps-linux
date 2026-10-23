//! Bridges the GLib main loop (UI) and a tokio runtime (HTTP).
//!
//! `reqwest` needs tokio, and the UI must never block, so API work runs on
//! a tokio worker pool and its result comes back to the main thread.

use std::future::Future;
use std::sync::OnceLock;

use gtk::glib;

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("substratal-io")
            .build()
            .expect("tokio runtime")
    })
}

/// Runs `fut` on the I/O runtime and hands its output to `done` on the
/// main thread.
pub fn spawn<F, T>(fut: F, done: impl FnOnce(T) + 'static)
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handle = runtime().spawn(fut);
    glib::spawn_future_local(async move {
        match handle.await {
            Ok(v) => done(v),
            Err(e) => eprintln!("substratal: background task failed: {e}"),
        }
    });
}

/// Blocks on `fut`. Only for start-up, before the first window exists.
pub fn block_on<F: Future>(fut: F) -> F::Output {
    runtime().block_on(fut)
}
