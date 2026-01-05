//! IPC server for client communication.

mod handlers;
mod server;

pub use server::{broadcast_event, get_event_sender, run_server};
