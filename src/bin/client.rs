use std::{net::{TcpStream}};
use std::io;
use redis::*;

//use redis::{send_req,read_res};

fn main() {
    let client = TcpStream::connect("0.0.0.0:1234");
    if let Err(ref e) = client {
        println!("Couldn't connect: {}", e);
        return;
    }
    let mut client = client.unwrap();

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() == "exit" {
            break;
        }

        if !send_req(&mut client, &input) {
            return;
        }

        if !read_res(&mut client) {
            return;
        }

        println!("Response received");
    }

    // let query_list = ["hello1", "hello2", "hello3"];
    //
    // for query in query_list {
    //    if !send_req(&mut client, query) {
    //        return;
    //    }
    // }
    //
    // for _ in 0..3 {
    //     if !read_res(&mut client) {
    //         return;
    //     }
    // }



    // let mut client = client.ok().unwrap();
    // println!("Connected to the server!");
    // let succ = query(&mut client, "hello1");
    // if !succ {
    //     return;
    // }
    // let succ = query(&mut client, "hello2");
    // if !succ {
    //     return;
    // }
    // let succ = query(&mut client, "hello3");
    // if !succ {
    //     return;
    //}
    // client.write(b"hello").unwrap();
    //
    // let mut buf: [u8; 128] = [0; 128];
    // let n = client.read(&mut buf).unwrap();
    // println!("Read {} bytes", n);
    // println!("Server says {}", String::from_utf8_lossy(&buf));
}
