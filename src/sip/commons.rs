use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{oneshot, Mutex};

pub struct TcpIpv4Client {
    pub rx_channel: Mutex<OwnedReadHalf>,
    pub tx_channel: Mutex<OwnedWriteHalf>,
    pub shutdown_signal_tx: Mutex<oneshot::Sender<()>>,
    pub shutdown_signal_rx: Mutex<oneshot::Receiver<()>>,
}
