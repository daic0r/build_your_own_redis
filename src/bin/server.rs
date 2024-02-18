use std::net::{TcpListener,SocketAddr,TcpStream};
use std::io::{Error, Read, Write};

use redis::one_request;

fn do_something(mut client: (TcpStream, SocketAddr)) {
    let mut buf: [u8; 128] = [0; 128];
    let n = client.0.read(&mut buf).unwrap();
    println!("Read {} bytes", n);
    println!("Client says {}", String::from_utf8_lossy(&buf));
    client.0.write(b"world").unwrap();
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:1234");
    if let Err(ref e) = listener {
        println!("Couldn't bind: {}", e.to_string());
        return;
    }
    let listener = listener.ok().unwrap();

    loop {
        let client = listener.accept();
        if let Err(ref e) = client {
            println!("Error accepting connection: {}", e.to_string());
            return;
        }
        let mut client = client.unwrap();
        client.0.set_read_timeout(Some(std::time::Duration::from_millis(500)));
        println!("Got a connection from {}", client.1);
        loop {
            let succ = one_request(&mut client.0);
            if !succ {
                break;
            }
        }
    }
}
