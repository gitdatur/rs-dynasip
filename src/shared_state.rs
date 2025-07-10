use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct SharedState {
    tcp_ipv4_servers: Mutex<HashMap<String, Arc<TcpListener>>>,
    tcp_ipv6_servers: Mutex<HashMap<String, Arc<TcpListener>>>,
    ipv4_tcp_clients_sockets: Mutex<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>,
    ipv6_tcp_clients_sockets: Mutex<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            tcp_ipv4_servers: Mutex::new(HashMap::new()),
            tcp_ipv6_servers: Mutex::new(HashMap::new()),
            ipv4_tcp_clients_sockets: Mutex::new(HashMap::new()),
            ipv6_tcp_clients_sockets: Mutex::new(HashMap::new()),
        }
    }

    pub async fn put_tcp_ipv4_server(&self, socket_addr: String, listener: Arc<TcpListener>) {
        let mut mutex_guard = self.tcp_ipv4_servers.lock().await;
        mutex_guard.insert(socket_addr, listener);
        drop(mutex_guard);
    }
    pub async fn put_ipv6_tcp_server(&self, socket_addr: String, listener: Arc<TcpListener>) {
        let mut mutex_guard = self.tcp_ipv6_servers.lock().await;
        mutex_guard.insert(socket_addr, listener);
        drop(mutex_guard);
    }
    pub async fn put_tcp_client_socket(
        &self,
        socket_addr: SocketAddr,
        tcp_stream: TcpStream,
    ) -> Arc<Mutex<TcpStream>> {
        let start = Instant::now();
        let tcp_stream = Arc::new(Mutex::new(tcp_stream));
        if socket_addr.is_ipv4() {
            let mut mutex_guard = self.ipv4_tcp_clients_sockets.lock().await;
            mutex_guard.insert(socket_addr, tcp_stream.clone());
            drop(mutex_guard);
        } else if socket_addr.is_ipv6() {
            let mut mutex_guard = self.ipv6_tcp_clients_sockets.lock().await;
            mutex_guard.insert(socket_addr, tcp_stream.clone());
            drop(mutex_guard);
        }
        println!("put_tcp_server took {:?}", start.elapsed());
        tcp_stream
    }
}
