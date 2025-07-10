mod shared_state;

use crate::shared_state::SharedState;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::combinator::recognize;
use nom::error::Error;
use nom::Parser;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

async fn process_socket(stream: Arc<Mutex<TcpStream>>) {
    let max_sip_message_size = 65536;
    let crlf_pattern = b"\r\n\r\n" as &[u8];
    let mut buffer = [0; 1421];
    let mut buffer_accumulator: Vec<u8> = Vec::with_capacity(max_sip_message_size);
    let mut last_parsed_index: usize = 0;
    loop {
        match stream.lock().await.read(&mut buffer).await {
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

#[tokio::main]
async fn main() -> io::Result<()> {
    let global_shard_state = Arc::new(SharedState::new());
    let local_socket_addr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::from_str("127.0.0.1").unwrap(),
        5060,
    ));

    let local_ipv4_server = global_shard_state
        .clone()
        .put_tcp_server(
            local_socket_addr,
            TcpListener::bind(local_socket_addr).await?,
        )
        .await;
    loop {
        let (stream, socket_address) = local_ipv4_server.accept().await?;
        let stream = global_shard_state
            .put_tcp_client_socket(socket_address, stream)
            .await;
        process_socket(stream).await;
    }
}
