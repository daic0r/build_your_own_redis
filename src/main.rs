use std::net::{TcpListener,SocketAddr,TcpStream};
use std::io::{Error, Read};

fn do_something(mut client: (TcpStream, SocketAddr)) {
    let mut buf: [u8; 128] = [0; 128];
    let n = client.0.read(&mut buf).unwrap();
    println!("Read {} bytes", n);
    println!("Data: {}", String::from_utf8_lossy(&buf));
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
        do_something(client.unwrap());

    }
}
