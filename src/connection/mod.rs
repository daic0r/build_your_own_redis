use std::{io::{Read, Write}, net::{TcpStream}};
use crate::database::Database;

use super::ResponseStatus;

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
    pub wbuf: [u8; 4 + MAX_MSG],
    pub database: Database,
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

        println!("Received {} bytes", rv);

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
            println!("Not enough data yet, need {}, have {}", 4 + len, self.rbuf_size);
            return false;
        }

        let msg = String::from_utf8_lossy(&self.rbuf[4..4+len]);
        println!("Client says: {}", msg);

        // self.wbuf[..4].copy_from_slice((len as u32).to_le_bytes().as_slice());
        // self.wbuf[4..4 + len].copy_from_slice(&self.rbuf[4..4 + len]);
        // self.wbuf_size = 4 + len;

        let mut res_len = 0usize;
        let status = Connection::do_request(&mut self.database, &self.rbuf[4..], len, &mut self.wbuf[4 + 4..], &mut res_len);
        if status == ResponseStatus::Err {
            eprintln!("Error processing request");
            self.state = ConnectionState::StateEnd;
            return false;
        }
        res_len += 4;
        self.wbuf[0..4].copy_from_slice(&(res_len as u32).to_le_bytes());
        self.wbuf[4..8].copy_from_slice(&(status as u32).to_le_bytes());
        self.wbuf_size = res_len + 4;

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

    fn parse_req(data: &[u8], len: usize) -> Option<Vec<String>> {
        if len < 4 {
            return None;
        }

        let mut num_commands = u32::from_le_bytes(data[0..4].try_into().expect("need 4-byte array")) as usize;

        let mut ret = vec![];

        let mut pos = 4;
        while num_commands > 0 {
            if pos + 4 > len {
                return None;
            }

            let len_arg = u32::from_le_bytes(data[pos..pos+4].try_into().expect("need 4-byte array")) as usize;
            if len_arg + 4 + pos > len {
                return None;
            }
            let arg = String::from_utf8_lossy(&data[pos+4..pos+4+len_arg]);

            ret.push(arg.to_string());

            num_commands -= 1;
            pos += 4 + len_arg;
        }

        if pos != len {
            return None;
        }

        Some(ret)
    }

    fn do_request(database: &mut Database, data: &[u8], len: usize, res_buf: &mut [u8], res_len: &mut usize) -> ResponseStatus {
        let args = Connection::parse_req(data, len);
        if args.is_none() {
            eprintln!("bad request");
            return ResponseStatus::Err;
        }
        let mut args = args.unwrap();

        assert!(args.len() > 1);

        match args.first().unwrap().as_str() {
            "get" => {
                if args.len() != 2 {
                    eprintln!("Invalid number of arguments for get command");
                    return ResponseStatus::Err;
                }
                println!("COMMAND: get {}", args[1]);
                let mut value = String::new();
                let ret = database.get(&args[1], &mut value);
                let bytes = value.as_bytes();
                res_buf[..bytes.len()].copy_from_slice(bytes);
                *res_len = bytes.len(); 
                return ret;
            },
            "set" => {
                if args.len() != 3 {
                    eprintln!("Invalid number of arguments for set command");
                    return ResponseStatus::Err;
                }
                println!("COMMAND: set {}={}", args[1], args[2]);
                let ret = database.set(std::mem::take(&mut args[1]), std::mem::take(&mut args[2]));
                return ret;
            },
            "del" => {
                if args.len() != 2 {
                    eprintln!("Invalid number of arguments for del command");
                    return ResponseStatus::Err;
                }
                println!("COMMAND: del {}", args[1]);
                let ret = database.del(&args[1]);
                return ret;
            },
            x => {
                eprintln!("Unknown command: {}", x);
            }
        }

        ResponseStatus::Err
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_req() {
        let mut buf = Vec::<u8>::new();
        buf.resize(50, u8::default());

        let num_args = 2u32.to_le_bytes();
        buf[0..4].copy_from_slice(&num_args);

        let arg1 = "hello".as_bytes();
        let len_arg1 = arg1.len() as u32;
        buf[4..8].copy_from_slice(&len_arg1.to_le_bytes());
        buf[8..8+arg1.len()].copy_from_slice(arg1);

        let start2 = 8 + arg1.len();
        let arg2 = "worlds".as_bytes();
        let len_arg2 = arg2.len() as u32;
        buf[start2..start2+4].copy_from_slice(&len_arg2.to_le_bytes());
        buf[start2+4..start2+4+arg2.len()].copy_from_slice(arg2);

        let res = Connection::parse_req(&buf, start2 + 4 + arg2.len());

        assert!(res.is_some()); 
        let res = res.unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], "hello");
        assert_eq!(res[1], "worlds");
    }

    #[test]
    fn test_do_request() {
        let mut buf = Vec::<u8>::new();
        buf.resize(50, u8::default());

        // For the resonse
        let mut res_buf = Vec::<u8>::new();
        res_buf.resize(MAX_MSG + 4, u8::default());
        let mut res_len = 0usize;


        let mut database = Database::new();



        //////////////////////////////////////////////////////////////////////
        let num_args = 3u32.to_le_bytes();
        buf[0..4].copy_from_slice(&num_args);

        let arg1 = "set".as_bytes();
        let len_arg1 = arg1.len() as u32;
        buf[4..8].copy_from_slice(&len_arg1.to_le_bytes());
        buf[8..8+arg1.len()].copy_from_slice(arg1);

        let start2 = 8 + arg1.len();
        let arg2 = "hello".as_bytes();
        let len_arg2 = arg2.len() as u32;
        buf[start2..start2+4].copy_from_slice(&len_arg2.to_le_bytes());
        buf[start2+4..start2+4+arg2.len()].copy_from_slice(arg2);

        let start3 = start2 + 4 + arg2.len();
        let arg3 = "world".as_bytes();
        let len_arg3 = arg3.len() as u32;
        buf[start3..start3+4].copy_from_slice(&len_arg3.to_le_bytes());
        buf[start3+4..start3+4+arg3.len()].copy_from_slice(arg3);

        let status = Connection::do_request(&mut database, &buf, start3 + 4 + arg3.len(), &mut res_buf, &mut res_len);
        assert_eq!(status, ResponseStatus::Ok);

        //////////////////////////////////////////////////////////////////////
        let num_args = 2u32.to_le_bytes();
        buf[0..4].copy_from_slice(&num_args);

        let arg1 = "get".as_bytes();
        let len_arg1 = arg1.len() as u32;
        buf[4..8].copy_from_slice(&len_arg1.to_le_bytes());
        buf[8..8+arg1.len()].copy_from_slice(arg1);

        let start2 = 8 + arg1.len();
        let arg2 = "hello".as_bytes();
        let len_arg2 = arg2.len() as u32;
        buf[start2..start2+4].copy_from_slice(&len_arg2.to_le_bytes());
        buf[start2+4..start2+4+arg2.len()].copy_from_slice(arg2);

        let status = Connection::do_request(&mut database, &buf, start2 + 4 + arg2.len(), &mut res_buf, &mut res_len);
        let res = String::from_utf8_lossy(&res_buf[..res_len]);
        assert_eq!(res, "world");
        assert_eq!(status, ResponseStatus::Ok);
        //////////////////////////////////////////////////////////////////////


        let num_args = 2u32.to_le_bytes();
        buf[0..4].copy_from_slice(&num_args);

        let arg1 = "del".as_bytes();
        let len_arg1 = arg1.len() as u32;
        buf[4..8].copy_from_slice(&len_arg1.to_le_bytes());
        buf[8..8+arg1.len()].copy_from_slice(arg1);

        let start2 = 8 + arg1.len();
        let arg2 = "hello".as_bytes();
        let len_arg2 = arg2.len() as u32;
        buf[start2..start2+4].copy_from_slice(&len_arg2.to_le_bytes());
        buf[start2+4..start2+4+arg2.len()].copy_from_slice(arg2);

        let status = Connection::do_request(&mut database, &buf, start2 + 4 + arg2.len(), &mut res_buf, &mut res_len);
        assert_eq!(status, ResponseStatus::Ok);
        //////////////////////////////////////////////////////////////////////

        let arg1 = "unknown".as_bytes();
        let len_arg1 = arg1.len() as u32;
        buf[4..8].copy_from_slice(&len_arg1.to_le_bytes());
        buf[8..8+arg1.len()].copy_from_slice(arg1);

        let status = Connection::do_request(&mut database, &buf, 8 + arg1.len(), &mut res_buf, &mut res_len);
        assert_eq!(status, ResponseStatus::Err);
    }
}
