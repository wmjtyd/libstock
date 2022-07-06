use nanomsg::{Socket, Protocol};

use std::io::{Write, Read};

pub struct Msg {
    socket: Socket
}

impl Msg {
    pub fn new(
        path: &str, 
        protocol: &str
    ) -> Msg {
        
        let mut socket = match protocol {
            "sub" => Socket::new(Protocol::Sub).unwrap(),
            "pub" => Socket::new(Protocol::Pub).unwrap(),
            _ => panic!("protocol error")
        };
            Socket::new(Protocol::Pub).unwrap();
        let _endpoint = socket
            .bind(path)
            .unwrap();
        Msg { socket }
    }
}

impl Send for Msg {
    fn send(&mut self, out: &[u8]) {
        self.socket.write_all(out).unwrap();
    }
}

impl Recv for Msg {
    fn recv(&mut self, out: &mut [u8]) {
        self.socket.read_exact(out).unwrap();
    }
}

pub trait Send {
    fn send(&mut self, out: &[u8]);
}

pub trait Recv {
    fn recv(&mut self, out: &mut [u8]);
}

pub fn publish(path: String) -> impl Recv {
    Msg::new(&path, "pub")
}

pub fn subscribe(path: String) -> impl Send {
    Msg::new(&path, "sub")
}
