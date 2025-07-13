use crate::sip::commons::TcpIpv4Client;
use crate::sip::SharedState;
use dashmap::mapref::one::Ref;
use log::{error, info};
use nom::bytes::complete::{tag, take_until};
use nom::combinator::recognize;
use nom::error::Error;
use nom::Parser;
use std::net::{AddrParseError, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Mutex};

pub enum Errors {
    InvalidIpv4Address,
    FailedToBindSocket,
    ServerAlreadyRunning,
}

pub struct TcpIpv4Server {
    shared_state: Arc<SharedState>,
    socket_address: SocketAddrV4,
    max_sip_message_size: usize,
    is_running: AtomicBool,
    shutdown_signal_rx: Mutex<oneshot::Receiver<()>>,
    shutdown_signal_tx: Mutex<oneshot::Sender<()>>,
}

impl TcpIpv4Server {
    pub fn new(
        shared_state: Arc<SharedState>,
        listen_address: String,
        listen_port: u16,
        max_sip_message_size: usize,
    ) -> Result<Arc<Self>, Errors> {
        let ipv4_address = Ipv4Addr::from_str(&*listen_address);
        let ipv4_address = match ipv4_address {
            Ok(val) => val,
            Err(_) => return Err(Errors::InvalidIpv4Address),
        };
        let socket_address: SocketAddrV4 = SocketAddrV4::new(ipv4_address, listen_port);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server = Self {
            shared_state,
            socket_address,
            max_sip_message_size,
            is_running: AtomicBool::new(false),
            shutdown_signal_rx: Mutex::new(shutdown_rx),
            shutdown_signal_tx: Mutex::new(shutdown_tx),
        };
        let server = Arc::new(server);
        server
            .shared_state
            .put_tcp_ipv4_server(socket_address, Arc::clone(&server));
        Ok(server)
    }

    pub async fn start(&self) -> Result<(), Errors> {
        if self.is_running.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Errors::ServerAlreadyRunning);
        }

        let tcp_listener_result = TcpListener::bind(self.socket_address).await;
        let tcp_listener = match tcp_listener_result {
            Ok(listener) => listener,
            Err(error) => {
                return Err(Errors::FailedToBindSocket);
            }
        };
        info!(
            "listening for tcp ipv4 connections on {}",
            self.socket_address
        );
        tokio::spawn(accept_clients(Arc::clone(&self.shared_state), tcp_listener));
        Ok(())
    }
}

async fn accept_clients(shared_state: Arc<SharedState>, tcp_listener: TcpListener) {
    loop {
        match tcp_listener.accept().await {
            Ok((stream, _)) => {
                info!(
                    "new tcp ipv4 client, local: {}, remote: {}",
                    stream.local_addr().unwrap(),
                    stream.peer_addr().unwrap()
                );
                let client_socket_address = stream.peer_addr();
                let client_socket_address: SocketAddr = match client_socket_address {
                    Ok(addr) => addr,
                    Err(_) => {
                        error!(
                            "failed to get client socket address to insert into clients map, terminating client connection"
                        );
                        drop(stream);
                        return;
                    }
                };
                let client_socket_address = match client_socket_address {
                    SocketAddr::V4(val) => val,
                    SocketAddr::V6(val) => {
                        error!(
                            "client socket address is of type v6 which shouldn't appear in a ipv4 server"
                        );
                        return;
                    }
                };
                let (shutdown_tx, shutdown_rx) = oneshot::channel();
                let (rx, tx) = stream.into_split();
                let tcp_ipv4_client = Arc::new(TcpIpv4Client {
                    rx_channel: Mutex::new(rx),
                    tx_channel: Mutex::new(tx),
                    shutdown_signal_tx: Mutex::new(shutdown_tx),
                    shutdown_signal_rx: Mutex::new(shutdown_rx),
                });
                shared_state
                    .put_tcp_ipv4_client(client_socket_address, Arc::clone(&tcp_ipv4_client));
                tokio::spawn(handle_client(
                    Arc::clone(&shared_state),
                    Arc::clone(&tcp_ipv4_client),
                ));
            }
            Err(_) => {}
        };
    }
}

async fn handle_client(shared_state: Arc<SharedState>, client: Arc<TcpIpv4Client>) {
    let mut rx = client.rx_channel.lock().await;
    let server_socket_address = match rx.local_addr().unwrap() {
        SocketAddr::V4(val) => val,
        SocketAddr::V6(val) => {
            error!(
                "local address (the server address the client connected to) of a tcp ipv4 client is an ipv6, which shouldn't happen"
            );
            return;
        }
    };
    let server = shared_state.get_tcp_ipv4_server(&server_socket_address);
    let server = match server {
        Some(server) => server,
        _ => {
            error!(
                "couldn't find a server with the local address found on the client socket, exiting!"
            );
            return;
        }
    };
    let max_sip_message_size = server.max_sip_message_size.clone();
    let crlf_pattern = b"\r\n\r\n" as &[u8];
    let mut buffer = [0; 1460];
    let mut buffer_accumulator: Vec<u8> = Vec::with_capacity(max_sip_message_size);
    let mut last_parsed_index: usize = 0;

    let client_socket_address = rx.peer_addr().unwrap();
    let client_socket_address = match client_socket_address {
        SocketAddr::V4(val) => val,
        SocketAddr::V6(val) => {
            error!(
                "client socket address (peer_addr of client socket) is of type ipv6 which should appear in ipv4 server, exiting!"
            );
            return;
        }
    };
    loop {
        match rx.read(&mut buffer).await {
            Ok(0) => {
                info!("client disconnected [{}]", rx.peer_addr().unwrap());
                shared_state.remove_tcp_ipv4_client(&client_socket_address);
                break;
            }
            Ok(number_of_read_bytes) => {
                if number_of_read_bytes + buffer_accumulator.len() >= max_sip_message_size {
                    error!(
                        "Too many bytes received and no CRLF reached, terminating client [{}]",
                        rx.peer_addr().unwrap()
                    );
                    break;
                }

                buffer_accumulator.extend_from_slice(&buffer[..number_of_read_bytes]);

                let parse_result = recognize((
                    take_until::<&[u8], &[u8], Error<&[u8]>>(crlf_pattern),
                    tag(crlf_pattern),
                ))
                .parse(
                    &buffer_accumulator[last_parsed_index.saturating_sub(5)
                        ..last_parsed_index + number_of_read_bytes - 1],
                );

                match parse_result {
                    Ok((untouched, touched)) => {
                        let potential_sip_message = buffer_accumulator.drain(
                            ..last_parsed_index + number_of_read_bytes - untouched.len() - 1,
                        );
                        let sip_parsing_result =
                            rsip::Request::try_from(potential_sip_message.as_slice());
                        match sip_parsing_result {
                            Ok(request) => {
                                info!(
                                    r#"
parsed a sip request from tcp ipv4 client {}
sip message
================================================
{}
================================================
"#,
                                    rx.peer_addr().unwrap(),
                                    request
                                )
                            }
                            Err(sip_parse_error) => {
                                error!(
                                    "failed to parse a sip message from tcp ipv4 client {} [{}]",
                                    rx.peer_addr().unwrap(),
                                    sip_parse_error.to_string()
                                )
                            }
                        }
                        last_parsed_index = 0;
                    }
                    Err(_) => {
                        last_parsed_index = last_parsed_index + number_of_read_bytes;
                    }
                }
            }
            Err(_) => {
                error!(
                    "failed to read from tcp client [{}]",
                    rx.peer_addr().unwrap()
                );
            }
        }
    }
}
