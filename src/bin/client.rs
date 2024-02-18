use std::{io::{Read, Write}, net::{TcpStream, SocketAddr}};

use redis::query;

fn main() {
    let client = TcpStream::connect("0.0.0.0:1234");
    if let Err(ref e) = client {
        println!("Couldn't connect: {}", e.to_string());
        return;
    }

    let mut client = client.ok().unwrap();
    println!("Connected to the server!");
    let succ = query(&mut client, "hello1");
    if !succ {
        return;
    }
    let succ = query(&mut client, "hello2");
    if !succ {
        return;
    }
    let succ = query(&mut client, "hello3");
    if !succ {
        return;
    }
    // client.write(b"hello").unwrap();
    //
    // let mut buf: [u8; 128] = [0; 128];
    // let n = client.read(&mut buf).unwrap();
    // println!("Read {} bytes", n);
    // println!("Server says {}", String::from_utf8_lossy(&buf));
}
