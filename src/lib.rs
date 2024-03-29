use std::{io::{Read, Write}, net::{TcpStream}};

pub const MAX_MSG: usize = 4096usize;

#[derive(PartialEq)]
pub enum ConnectionState {
    StateReq,
    StateRes,
    StateEnd
}

pub struct Connection {
    pub fd: TcpStream,
    pub state: ConnectionState,
    pub rbuf_size: usize,
    pub rbuf: [u8; 4 + MAX_MSG],
    pub wbuf_size: usize,
    pub wbuf_sent: usize,
    pub wbuf: [u8; 4 + MAX_MSG]
}

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

pub fn one_request(stream: &mut TcpStream) -> bool {

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
    println!("Client says: {}", msg);

    let reply = "world";
    let len_as_bytes = (reply.len() as u32).to_le_bytes();
    buf[0..4].copy_from_slice(&len_as_bytes);
    buf[4..4+reply.len()].copy_from_slice(reply.as_bytes());
    let ret = write_all(stream, &buf, 4 + reply.len());

    ret
}

pub fn query(stream: &mut TcpStream, text: &str) -> bool {
    let mut buf: [u8; 4 + MAX_MSG] = [0; 4 + MAX_MSG];

    let len_as_bytes = (text.len() as u32).to_le_bytes();
    buf[0..4].copy_from_slice(&len_as_bytes);
    buf[4..4+text.len()].copy_from_slice(text.as_bytes());
    write_all(stream, &buf, 4 + text.len());

    buf.fill_with(Default::default);

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
