use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{oneshot, Mutex};

pub struct SocketProperties {
    pub max_sip_message_size: usize,
    pub buffer_size: usize,
    pub ip_address: String,
    pub port: u16,
}

pub struct TcpClient {
    pub rx: Mutex<OwnedReadHalf>,
    pub tx: Mutex<OwnedWriteHalf>,
    pub shutdown_signal: Mutex<oneshot::Sender<()>>,
}
