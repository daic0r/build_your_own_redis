use std::net::TcpStream;
use std::io::{Read, Write};

#[derive(PartialEq, Debug)]
pub enum ResponseStatus {
    Ok,
    Err,
    Nx
}

pub mod connection;
pub mod database;

use crate::connection::MAX_MSG;

//use std::{io::{Read, Write}, net::{TcpStream}};

pub fn read_full(stream: &mut TcpStream, buf: &mut [u8], n: usize) -> bool {
    let mut bytes_left = Some(n);
    let mut offset = 0;
    loop {
        let bytes = stream.read(&mut buf[offset..offset + bytes_left.unwrap()]);
        if bytes.is_err() || *bytes.as_ref().unwrap() == 0 {
            return false;
        }
        bytes_left = bytes_left.unwrap().checked_sub(*bytes.as_ref().unwrap());
        if bytes_left.is_none() || bytes_left.unwrap() == 0 {
            break;
        }
        offset += bytes.unwrap();
    }
    true
}

pub fn write_all(stream: &mut TcpStream, buf: &[u8], n: usize) -> bool {
    let mut bytes_left = Some(n);
    let mut offset = 0;
    while bytes_left.is_some() && bytes_left.unwrap() > 0 {
        let bytes = stream.write(&buf[offset..offset + bytes_left.unwrap()]);
        if bytes.is_err() {
            return false;
        }
        bytes_left = bytes_left.unwrap().checked_sub(*bytes.as_ref().unwrap());
        offset += bytes.unwrap();
    }
    true
}
//
// pub fn one_request(stream: &mut TcpStream) -> bool {
//
//     let mut buf: [u8; 4 + MAX_MSG] = [0; 4 + MAX_MSG];
//
//     let err = read_full(stream, &mut buf, 4);
//     if !err {
//         eprintln!("Error reading buffer length");
//         return false;
//     }
//
//     let len = u32::from_le_bytes(buf[0..4].try_into().expect("Must be a 4 byte array"));
//     let err = read_full(stream, &mut buf[4..], len as usize);
//     if !err {
//         eprintln!("Error reading message");
//         return false;
//     }
//     let msg = String::from_utf8_lossy(&buf[4..]);
//     println!("Client says: {}", msg);
//
//     let reply = "world";
//     let len_as_bytes = (reply.len() as u32).to_le_bytes();
//     buf[0..4].copy_from_slice(&len_as_bytes);
//     buf[4..4+reply.len()].copy_from_slice(reply.as_bytes());
//
//
//     write_all(stream, &buf, 4 + reply.len())
// }
//
pub fn send_req(stream: &mut TcpStream, text: &str) -> bool {
    let mut buf: [u8; 4 + MAX_MSG] = [0; 4 + MAX_MSG];

    let len_as_bytes = (text.len() as u32).to_le_bytes();
    buf[0..4].copy_from_slice(&len_as_bytes);
    buf[4..4+text.len()].copy_from_slice(text.as_bytes());

    write_all(stream, &buf, 4 + text.len())
}

pub fn read_res(stream: &mut TcpStream) -> bool {
    let mut buf: [u8; 4 + MAX_MSG] = [0; 4 + MAX_MSG];

    let err = read_full(stream, &mut buf, 4);
    if !err {
        eprintln!("Error reading buffer length");
        return false;
    }

    let len = u32::from_le_bytes(buf[0..4].try_into().expect("Must be a 4 byte array"));
    let err = read_full(stream, &mut buf[4..], len as usize);
    if !err {
        eprintln!("Error reading message");
        return false;
    }

    let msg = String::from_utf8_lossy(&buf[4..]);
    println!("Server says: {}", msg);

    true
}
//
