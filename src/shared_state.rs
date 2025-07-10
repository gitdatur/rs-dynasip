use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct SharedState {
    ipv4_tcp_servers_sockets: Mutex<HashMap<SocketAddr, Arc<TcpListener>>>,
    ipv6_tcp_servers_sockets: Mutex<HashMap<SocketAddr, Arc<TcpListener>>>,
    ipv4_tcp_clients_sockets: Mutex<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>,
    ipv6_tcp_clients_sockets: Mutex<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            ipv4_tcp_servers_sockets: Mutex::new(HashMap::new()),
            ipv6_tcp_servers_sockets: Mutex::new(HashMap::new()),
            ipv4_tcp_clients_sockets: Mutex::new(HashMap::new()),
            ipv6_tcp_clients_sockets: Mutex::new(HashMap::new()),
        }
    }

    pub async fn put_tcp_server(
        &self,
        socket_addr: SocketAddr,
        listener: TcpListener,
    ) -> Arc<TcpListener> {
        let start = Instant::now();
        let arc_holder = Arc::new(listener);
        if socket_addr.is_ipv4() {
            let mut mutex_guard = self.ipv4_tcp_servers_sockets.lock().await;
            mutex_guard.insert(socket_addr, arc_holder.clone());
            drop(mutex_guard);
        } else if socket_addr.is_ipv6() {
            let mut mutex_guard = self.ipv6_tcp_servers_sockets.lock().await;
            mutex_guard.insert(socket_addr, arc_holder.clone());
            drop(mutex_guard);
        }
        println!("put_tcp_server took {:?}", start.elapsed());
        arc_holder
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
