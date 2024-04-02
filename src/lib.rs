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

impl Connection {
    pub fn state_req(&mut self) {
        while self.try_fill_buffer() {
            println!("state_req looping");
        }
    }

    fn try_fill_buffer(&mut self) -> bool {
        assert!(self.rbuf_size < self.rbuf.len());

        let mut rv = 0usize;
        loop {
            match self.fd.read(&mut self.rbuf[self.rbuf_size..]) {
                Ok(n) => {
                    rv = n;
                    break;
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return false;
                },
                Err(_) => {
                    eprintln!("read() error");
                    self.state = ConnectionState::StateEnd;
                    return false;
                }
            }
        }

        if rv == 0 {
            if self.rbuf_size > 0 {
                eprintln!("Unexpected EOF");
            } else {
                eprintln!("EOF");
            }
            self.state = ConnectionState::StateEnd;
            return false;
        }

        self.rbuf_size += rv;
        assert!(self.rbuf_size <= self.rbuf.len());

        while self.try_one_request() {}

        self.state == ConnectionState::StateReq
    }

    fn try_one_request(&mut self) -> bool {
        if self.rbuf_size < 4 {
            return false;
        }

        let len = u32::from_le_bytes(self.rbuf[0..4].try_into().expect("need 4-byte array")) as usize;
        if len > MAX_MSG {
            eprintln!("Message too long");
            self.state = ConnectionState::StateEnd;
            return false;
        }
        if 4 + len > self.rbuf_size {
            return false;
        }

        let msg = String::from_utf8_lossy(&self.rbuf[4..4+len]);
        println!("Client says: {}", msg);

        self.wbuf[..4].copy_from_slice((len as u32).to_le_bytes().as_slice());
        self.wbuf[4..4 + len].copy_from_slice(&self.rbuf[4..4 + len]);
        self.wbuf_size = 4 + len;

        let remain = self.rbuf_size - 4 - len;
        if remain > 0 {
            self.rbuf.copy_within(4 + len.., 0);
        }
        self.rbuf_size = remain;
        self.state = ConnectionState::StateRes;
        self.state_res();

        self.state == ConnectionState::StateReq
    }

    pub fn state_res(&mut self) {
        while self.try_flush_buffer() {
            println!("state_res looping");
        }
    }

    fn try_flush_buffer(&mut self) -> bool {
        let mut rv = 0usize;
        loop {
            let remain = self.wbuf_size - self.wbuf_sent;
            match self.fd.write(&mut self.wbuf[self.wbuf_sent..self.wbuf_sent+remain]) {
                Ok(n) => {
                    rv = n;
                    break;
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return false;
                },
                Err(_) => {
                    eprintln!("write() error");
                    self.state = ConnectionState::StateEnd;
                    return false;
                }
            }
        }

        self.wbuf_sent += rv;
        assert!(self.wbuf_sent <= self.wbuf_size);

        if self.wbuf_sent == self.wbuf_size {
            self.state = ConnectionState::StateReq;
            self.wbuf_sent = 0;
            self.wbuf_size = 0;
            return false;
        }

        true
    }
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
