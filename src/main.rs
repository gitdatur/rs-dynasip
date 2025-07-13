mod sip;

use crate::sip::servers::tcp_ipv4::TcpIpv4Server;
use crate::sip::SharedState;
use log::error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWrite};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let shared_state = Arc::new(SharedState::new());
    let localhost_tcp_ipv4_server = TcpIpv4Server::new(
        Arc::clone(&shared_state),
        "127.0.0.1".to_string(),
        5060,
        65000,
    );
    let localhost_tcp_ipv4_server = match localhost_tcp_ipv4_server {
        Ok(server) => server,
        Err(error) => {
            error!("failed to start ipv4 server on localhost");
            return;
        }
    };
    match localhost_tcp_ipv4_server.start().await {
        Ok(_) => {}
        Err(_) => {}
    };
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}
