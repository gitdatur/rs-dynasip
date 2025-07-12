use crate::sip::common_structs::TcpClient;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct SharedState {
    tcp_ipv4_servers: DashMap<String, Arc<TcpListener>>,
    tcp_ipv6_servers: DashMap<String, Arc<TcpListener>>,
    tcp_ipv4_clients: DashMap<String, Arc<TcpClient>>,
    tcp_ipv6_clients: DashMap<String, Arc<TcpClient>>,
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

    pub async fn put_tcp_ipv4_server(&self, socket_addr: String, listener: Arc<TcpListener>) {
        self.tcp_ipv4_servers.insert(socket_addr, listener);
    }
    pub async fn put_ipv6_tcp_server(&self, socket_addr: String, listener: Arc<TcpListener>) {
        self.tcp_ipv6_servers.insert(socket_addr, listener);
    }
    pub async fn put_tcp_ipv4_client(&self, socket_addr: String, tcp_client: Arc<TcpClient>) {
        self.tcp_ipv4_clients.insert(socket_addr, tcp_client);
    }
    pub async fn get_tcp_ipv4_client(
        &self,
        socket_addr: &String,
    ) -> Option<Ref<String, Arc<TcpClient>>> {
        self.tcp_ipv4_clients.get(socket_addr)
    }
    pub async fn put_ipv6_tcp_client(&self, socket_addr: String, tcp_client: Arc<TcpClient>) {
        self.tcp_ipv6_clients.insert(socket_addr, tcp_client);
    }

    pub async fn remove_tcp_ipv4_client(
        &self,
        socket_addr: &String,
    ) -> Option<(String, Arc<TcpClient>)> {
        self.tcp_ipv4_clients.remove(socket_addr)
    }
}
