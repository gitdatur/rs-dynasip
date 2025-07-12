mod shared_state;
mod sip;

use crate::shared_state::SharedState;
use crate::sip::common_structs::SocketProperties;
use crate::sip::tcp_ipv4_server::TcpIpv4Server;
use nom::Parser;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let global_shared_state = Arc::new(SharedState::new());
    let localhost_tcp_ipv4_server = TcpIpv4Server::new(
        Arc::clone(&global_shared_state),
        SocketProperties {
            max_sip_message_size: 65536,
            buffer_size: 1412,
            ip_address: "127.0.0.1".to_string(),
            port: 5060,
        },
    );
    match localhost_tcp_ipv4_server.start().await {
        Ok(_) => {}
        Err(_) => {}
    };
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}
