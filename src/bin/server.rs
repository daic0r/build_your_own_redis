use std::io::{Error, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

use redis::*;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1234");
    if let Err(ref e) = listener {
        println!("Couldn't bind: {}", e.to_string());
        return;
    }
    let listener = listener.ok().unwrap();
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    let mut connections = vec![];

    loop {
        match listener.accept() {
            Ok((client, addr)) => {
                client
                    .set_nonblocking(true)
                    .expect("Couldn't set non-blocking mode on accepted connection");
                println!("Got a connection from {}", addr);
                let conn = Connection {
                    fd: client,
                    state: ConnectionState::StateReq,
                    rbuf_size: 0,
                    rbuf: [0; 4 + MAX_MSG],
                    wbuf_size: 0,
                    wbuf_sent: 0,
                    wbuf: [0; 4 + MAX_MSG],
                };
                connections.push(conn);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
        for client in &mut connections {
            match client.state {
                ConnectionState::StateReq => {
                    client.state_req();
                },
                ConnectionState::StateRes => {
                    client.state_res();
                }
                _ => {}
            }
        }
        connections.retain(|e| { 
            if e.state == ConnectionState::StateEnd {
                println!("Client disconnected");
            }
            e.state != ConnectionState::StateEnd
        });
    }
}
