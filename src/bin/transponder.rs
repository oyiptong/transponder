#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate tokio_core;

use std::{env, io};
use std::error::Error;
use std::process::exit;
use std::net::SocketAddr;

use futures::{Future, Poll};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;


pub fn unexpected_error<T>(err :T) -> !
    where T: std::error::Error,
{
    println!("Failure: {}", err.description().to_string());
    exit(1);
}

pub fn unexpected_io_error(err :std::io::Error) -> ! {
    println!("Failure: {}", err.description().to_string());
    match err.raw_os_error() {
        Some(code) => exit(code),
        None => exit(1),
    }
}

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

struct UDPServer<'a> {
    ioloop: tokio_core::reactor::Core,
    transponder: UDPTransponder,
    addr: &'a SocketAddr,
}

impl<'a> UDPServer<'a> {
    fn new(addr: &SocketAddr) -> Result<UDPServer, io::Error> {
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
            addr: addr,
        })
    }
    
    fn run(&mut self) -> Result<(), io::Error>{
        println!("Listening on: {} using UDP", self.addr);
        let transponder = &mut self.transponder;
        try!(self.ioloop.run(transponder));
        Ok(())
    }
}

fn main() {
    env_logger::init().expect("Unable to init logger");

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:48656".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap_or_else({|e| unexpected_error(e)});

    let udp_server = &mut UDPServer::new(&addr).unwrap_or_else({|e| unexpected_io_error(e)});
    udp_server.run().unwrap_or_else({|e| unexpected_io_error(e)});
}
