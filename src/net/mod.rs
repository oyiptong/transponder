extern crate tokio_core;
extern crate futures;

use std::io;
use std::net::SocketAddr;
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;
use futures::{Future, Poll};


struct UDPTransponder {
    socket: UdpSocket,
    buf: Vec<u8>,
    incoming: Option<(usize, SocketAddr)>,
}

impl Future for UDPTransponder {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            if let Some((size, peer)) = self.incoming {
                println!("RECEIVED: {} bytes FROM {}", size, peer);
                self.incoming = None;
            }

            self.incoming = Some(try_nb!(self.socket.recv_from(&mut self.buf)));
        }
    }
}

pub struct UDPServer {
    ioloop: tokio_core::reactor::Core,
    transponder: UDPTransponder,
}

impl UDPServer {
    pub fn new(addr: &SocketAddr) -> Result<UDPServer, io::Error> {
        let ioloop = try!(Core::new());
        let handle = ioloop.handle();
        let socket = try!(UdpSocket::bind(&addr, &handle));
        let server = UDPTransponder {
            socket: socket,
            buf: vec![0; 1024],
            incoming: None,
        };
        Ok(UDPServer {
            ioloop: ioloop,
            transponder: server,
        })
    }
    
    pub fn run(&mut self) -> Result<(), io::Error>{
        let transponder = &mut self.transponder;
        try!(self.ioloop.run(transponder));
        Ok(())
    }
}
