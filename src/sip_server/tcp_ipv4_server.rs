use crate::shared_state::SharedState;
use crate::sip_server::common_structs::SocketProperties;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct TcpIpv4Server {
    shared_state: std::sync::Arc<SharedState>,
    socket_properties: SocketProperties,
}

impl TcpIpv4Server {
    pub fn new(
        shared_state: std::sync::Arc<SharedState>,
        socket_properties: SocketProperties,
    ) -> Self {
        Self {
            shared_state,
            socket_properties,
        }
    }

    pub async fn start(&self) -> Result<(), &str> {
        let ipv4_result = Ipv4Addr::from_str(&*self.socket_properties.ip_address);
        let ipv4 = match ipv4_result {
            Ok(val) => val,
            Err(re) => {
                return Err("error parsing ip address");
            }
        };
        let socket_address = SocketAddr::V4(SocketAddrV4::new(ipv4, 5060));
        let tcp_listener_result = TcpListener::bind(socket_address).await;
        let tcp_listener = match tcp_listener_result {
            Ok(listener) => Arc::new(listener),
            Err(_) => {
                return Err("failed to listen on socket");
            }
        };
        self.shared_state
            .put_tcp_ipv4_server(socket_address.to_string(), Arc::clone(&tcp_listener))
            .await;
        tokio::spawn(accept_clients(
            Arc::clone(&tcp_listener),
            Arc::clone(&self.shared_state),
        ));
        Ok(())
    }
}

async fn accept_clients(tcp_listener: Arc<TcpListener>, shared_state: std::sync::Arc<SharedState>) {
    loop {
        match tcp_listener.accept().await {
            Ok((stream, _)) => {}
            Err(_) => {}
        };
    }
}

async fn process_tcp_stream() {}
