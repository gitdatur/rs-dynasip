use crate::sip_server::common_structs::TcpClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};

pub struct SharedState {
    tcp_ipv4_servers: RwLock<HashMap<String, Arc<TcpListener>>>,
    tcp_ipv6_servers: RwLock<HashMap<String, Arc<TcpListener>>>,
    tcp_ipv4_clients: RwLock<HashMap<String, Arc<RwLock<TcpClient>>>>,
    tcp_ipv6_clients: RwLock<HashMap<String, Arc<RwLock<TcpClient>>>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            tcp_ipv4_servers: RwLock::new(HashMap::new()),
            tcp_ipv6_servers: RwLock::new(HashMap::new()),
            tcp_ipv4_clients: RwLock::new(HashMap::new()),
            tcp_ipv6_clients: RwLock::new(HashMap::new()),
        }
    }

    pub async fn put_tcp_ipv4_server(&self, socket_addr: String, listener: Arc<TcpListener>) {
        let mut mutex_guard = self.tcp_ipv4_servers.write().await;
        mutex_guard.insert(socket_addr, listener);
        drop(mutex_guard);
    }
    pub async fn put_ipv6_tcp_server(&self, socket_addr: String, listener: Arc<TcpListener>) {
        let mut mutex_guard = self.tcp_ipv6_servers.write().await;
        mutex_guard.insert(socket_addr, listener);
        drop(mutex_guard);
    }
    pub async fn put_tcp_ipv4_client(
        &self,
        socket_addr: String,
        tcp_client: Arc<RwLock<TcpClient>>,
    ) {
        let mut mutex_guard = self.tcp_ipv4_clients.write().await;
        mutex_guard.insert(socket_addr, tcp_client);
        drop(mutex_guard);
    }
    pub async fn put_ipv6_tcp_client(
        &self,
        socket_addr: String,
        tcp_client: Arc<RwLock<TcpClient>>,
    ) {
        let mut mutex_guard = self.tcp_ipv6_clients.write().await;
        mutex_guard.insert(socket_addr, tcp_client);
        drop(mutex_guard);
    }
}
