use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex};

pub struct SocketProperties {
    pub max_sip_message_size: usize,
    pub buffer_size: usize,
    pub ip_address: String,
    pub port: u16,
}

pub struct TcpClient {
    pub(crate) stream: Mutex<TcpStream>,
    pub disconnect_client_signal: Mutex<oneshot::Sender<()>>,
}
