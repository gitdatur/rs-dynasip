use crate::shared_state::SharedState;
use crate::sip::common_structs::{SocketProperties, TcpClient};
use log::{error, info};
use nom::bytes::complete::{tag, take_until};
use nom::combinator::recognize;
use nom::error::Error;
use nom::Parser;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Mutex};

pub struct TcpIpv4Server {
    shared_state: Arc<SharedState>,
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
        info!("listening for tcp ipv4 connections on {}", socket_address);
        tokio::spawn(accept_clients(
            Arc::clone(&tcp_listener),
            Arc::clone(&self.shared_state),
        ));
        Ok(())
    }
}

async fn accept_clients(tcp_listener: Arc<TcpListener>, shared_state: Arc<SharedState>) {
    loop {
        match tcp_listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_client(stream, Arc::clone(&shared_state)));
            }
            Err(_) => {}
        };
    }
}

async fn handle_client(stream: TcpStream, shared_state: Arc<SharedState>) {
    info!(
        "new tcp ipv4 client, local: {}, remote: {}",
        stream.local_addr().unwrap(),
        stream.peer_addr().unwrap()
    );
    let client_address = stream.peer_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (rx, tx) = stream.into_split();
    let client = Arc::new(TcpClient {
        rx: Mutex::new(rx),
        tx: Mutex::new(tx),
        shutdown_signal: Mutex::new(shutdown_tx),
    });
    let mut rx = client.rx.lock().await;
    shared_state
        .put_tcp_ipv4_client(client_address.to_string(), Arc::clone(&client))
        .await;
    let max_sip_message_size = 65536;
    let crlf_pattern = b"\r\n\r\n" as &[u8];
    let mut buffer = [0; 1421];
    let mut buffer_accumulator: Vec<u8> = Vec::with_capacity(max_sip_message_size);
    let mut last_parsed_index: usize = 0;

    loop {
        match rx.read(&mut buffer).await {
            Ok(0) => {
                info!("client disconnected [{}]", rx.peer_addr().unwrap());
                shared_state
                    .remove_tcp_ipv4_client(&rx.peer_addr().unwrap().to_string())
                    .await;
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
                        let potential_sip_message = buffer_accumulator
                            .drain(..last_parsed_index + number_of_read_bytes - untouched.len());
                        let sip_parsing_result =
                            rsip::Request::try_from(potential_sip_message.as_slice());
                        match sip_parsing_result {
                            Ok(request) => {
                                info!(
                                    "parsed a sip message from tcp ipv4 client {}",
                                    rx.peer_addr().unwrap()
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
