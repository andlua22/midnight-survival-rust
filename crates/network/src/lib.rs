//! Encrypted transport shared by the graphical client, dedicated server and load runner.
pub mod client;
pub mod codec;
pub mod identity;
pub mod server;

pub use quinn;
pub use tokio;
pub type NetResult<T> = Result<T, String>;

pub fn runtime() -> NetResult<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}
