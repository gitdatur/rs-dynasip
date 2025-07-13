pub mod client_handlers;
pub mod commons;
pub mod dialogs_manager;
pub mod servers;
pub mod transactions_manager;

use crate::sip::commons::TcpIpv4Client;
use crate::sip::servers::tcp_ipv4::TcpIpv4Server;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use std::net::SocketAddrV4;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct SharedState {
    tcp_ipv4_servers: DashMap<SocketAddrV4, Arc<TcpIpv4Server>>,
    tcp_ipv6_servers: DashMap<String, Arc<TcpListener>>,
    tcp_ipv4_clients: DashMap<SocketAddrV4, Arc<TcpIpv4Client>>,
    tcp_ipv6_clients: DashMap<String, Arc<TcpIpv4Client>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            tcp_ipv4_servers: DashMap::new(),
            tcp_ipv6_servers: DashMap::new(),
            tcp_ipv4_clients: DashMap::new(),
            tcp_ipv6_clients: DashMap::new(),
        }
    }

    pub fn put_tcp_ipv4_server(&self, socket_addr: SocketAddrV4, server: Arc<TcpIpv4Server>) {
        self.tcp_ipv4_servers.insert(socket_addr, server);
    }
    pub fn put_tcp_ipv6_server(&self, socket_addr: String, listener: Arc<TcpListener>) {
        self.tcp_ipv6_servers.insert(socket_addr, listener);
    }
    pub fn put_tcp_ipv4_client(&self, socket_addr: SocketAddrV4, tcp_client: Arc<TcpIpv4Client>) {
        self.tcp_ipv4_clients.insert(socket_addr, tcp_client);
    }
    pub fn get_tcp_ipv4_server(
        &self,
        socket_addr: &SocketAddrV4,
    ) -> Option<Ref<SocketAddrV4, Arc<TcpIpv4Server>>> {
        self.tcp_ipv4_servers.get(socket_addr)
    }
    pub fn get_tcp_ipv4_client(
        &self,
        socket_addr: &SocketAddrV4,
    ) -> Option<Ref<SocketAddrV4, Arc<TcpIpv4Client>>> {
        self.tcp_ipv4_clients.get(socket_addr)
    }
    pub async fn put_ipv6_tcp_client(&self, socket_addr: String, tcp_client: Arc<TcpIpv4Client>) {
        self.tcp_ipv6_clients.insert(socket_addr, tcp_client);
    }

    pub fn remove_tcp_ipv4_client(
        &self,
        socket_addr: &SocketAddrV4,
    ) -> Option<(SocketAddrV4, Arc<TcpIpv4Client>)> {
        self.tcp_ipv4_clients.remove(socket_addr)
    }
}
