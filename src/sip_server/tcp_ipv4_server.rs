use crate::shared_state::SharedState;
use crate::sip_server::common_structs::{SocketProperties, TcpClient};
use nom::bytes::complete::{tag, take_until};
use nom::combinator::recognize;
use nom::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
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
                let (tx, rx) = oneshot::channel();
                let tcp_client = Arc::new(RwLock::new(TcpClient { stream: Mutex::new(stream), disconnect_client_signal: Mutex::new(tx) })));
                let tcp_client_guard = tcp_client.stream.lock().await;
                let socket_address = tcp_client_guard.peer_addr().unwrap().to_string();
                shared_state
                    .put_tcp_ipv4_client(socket_address, Arc::clone(&tcp_client))
                    .await;
                drop(tcp_client_guard);
                tokio::spawn(handle_tcp_client(Arc::clone(&tcp_client)));
            }
            Err(_) => {}
        };
    }
}

async fn handle_tcp_client(tcp_client: Arc<Mutex<TcpClient>>) {
    let max_sip_message_size = 65536;
    let crlf_pattern = b"\r\n\r\n" as &[u8];
    let mut buffer = [0; 1421];
    let mut buffer_accumulator: Vec<u8> = Vec::with_capacity(max_sip_message_size);
    let mut last_parsed_index: usize = 0;
    loop {
        match tcp_client.lock().await.read(&mut buffer).await {
            Ok(0) => {
                println!("Connection closed.");
                break;
            }
            Ok(number_of_read_bytes) => {
                println!("number_of_read_bytes {}", number_of_read_bytes);
                if number_of_read_bytes + buffer_accumulator.len() >= max_sip_message_size {
                    println!("Too many bytes received and no CRLF reached, terminating socket");
                    break;
                }

                buffer_accumulator.extend_from_slice(&buffer[..number_of_read_bytes]);
                println!("buffer_accumulator.len() : {}", buffer_accumulator.len());

                let parse_result = recognize((
                    take_until::<&[u8], &[u8], Error<&[u8]>>(crlf_pattern),
                    tag(crlf_pattern),
                ))
                    .parse(
                        &buffer_accumulator[last_parsed_index.saturating_sub(5)
                            ..last_parsed_index + number_of_read_bytes - 1],
                    );
                println!(
                    "parsing from {} to {}",
                    last_parsed_index.saturating_sub(5),
                    last_parsed_index + number_of_read_bytes - 1
                );

                match parse_result {
                    Ok((untouched, touched)) => {
                        println!("touched.len() : {}", touched.len());
                        println!("untouched.len() : {}", untouched.len());
                        println!("last_parsed_index {}", last_parsed_index);
                        println!(
                            "draining touched.len() and prev parsed depending on last parsed index [{}]",
                            touched.len() + last_parsed_index + number_of_read_bytes
                        );
                        let potential_sip_message = buffer_accumulator
                            .drain(..last_parsed_index + number_of_read_bytes - untouched.len());
                        let sip_parsing_result =
                            rsip::Request::try_from(potential_sip_message.as_slice());
                        match sip_parsing_result {
                            Ok(request) => {
                                println!("parsed sip message: {:?}", request);
                            }
                            Err(sip_parse_error) => {
                                println!("sip_parse_error: {:?}", sip_parse_error);
                            }
                        }
                        last_parsed_index = 0;
                    }
                    Err(_) => {
                        last_parsed_index = last_parsed_index + number_of_read_bytes;
                    }
                }
                println!("last_parsed_index {}", last_parsed_index);
            }
            Err(_) => {
                println!("failed to read data from socket")
            }
        }
    }
}
