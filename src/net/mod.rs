extern crate tokio_core;
extern crate futures;
extern crate net2;
extern crate error_type;
extern crate threadpool;
extern crate hyper;

use std::{io, thread, str};
use std::error::Error as StdError;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, Condvar};
use self::net2::unix::UnixUdpBuilderExt;
use self::net2::UdpBuilder;
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;
use futures::{Future, Poll};
use utils::{Config, Error};
use self::threadpool::ThreadPool;
use hyper::header::{Headers, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};


/// A server that forwards payloads from a UDP socket for processing
struct UDPReceiver {
    socket: UdpSocket,
    buf: Vec<u8>,
    incoming: Option<(usize, SocketAddr)>,
    sync: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>,
}

impl Future for UDPReceiver {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            self.incoming = Some(try_nb!(self.socket.recv_from(&mut self.buf)));

            if let Some((size, _)) = self.incoming {
                let data = &self.buf[..size];
                let byte_vec = data.to_vec();
                let &(ref lock, ref cvar) = &*self.sync;
                let mut items = lock.lock().expect("input failed to acquire lock");
                items.push(byte_vec);
                cvar.notify_all();
                self.incoming = None;
            }
        }
    }
}

/// Starts a worker that waits for JSON payloads to send
fn run_json_sender_worker(config: Config, sync: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>, client: Arc<hyper::Client>) -> Result<(), Error> {
    let &(ref lock, ref cvar) = &*sync;
    loop {
        let mut items = lock.lock().expect("client failed to acquire lock");

        if items.len() == 0 {
            items = cvar.wait(items).expect("client failed to wait")
        }

        match items.pop() {
            Some(item) => {
                info!("SENDING: {} bytes", item.len());

                let mut headers = Headers::new();
                headers.set(ContentType(Mime(
                    TopLevel::Application,
                    SubLevel::Json,
                    vec![(Attr::Charset, Value::Utf8)],
                )));

                let res = client.post(config.receiver_url.as_str())
                    .headers(headers)
                    .body(item.as_slice())
                    .send();

                match res {
                    Ok(response) => {
                        info!("RESPONSE: {}", response.status);
                    },
                    Err(e) => {
                        info!("REQUEST_ERROR: {}", e.description());
                    },
                }
            },
            None => {
                debug!("client: nothing to do");
            }
        }
    }
}

/// Starts an evented UDP message listener
/// It is assumed that the port being listened to is re-used
fn run_udp_receiver(config: Config, sync: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>) -> Result<(), Error> {
    let mut ioloop = try!(Core::new());
    let handle = ioloop.handle();

    let builder = try!(UdpBuilder::new_v4());

    match builder.reuse_address(true) {
        Ok(_) => debug!("enabled SO_REUSEADDR"),
        Err(e) => warn!("failed to enable SO_REUSEADDR: {}", e.description()),
    }

    match builder.reuse_port(true) {
        Ok(_) => debug!("enabled SO_REUSEPORT"),
        Err(e) => warn!("failed to enable SO_REUSEPORT: {}", e.description()),
    }

    let socket = try!(builder.bind(&config.addr));
    let evented_socket = try!(UdpSocket::from_socket(socket, &handle));

    let receiver = UDPReceiver {
        socket: evented_socket,
        buf: vec![0; config.mtu as usize],
        incoming: None,
        sync: sync,
    };

    try!(ioloop.run(receiver));
    Ok(())
}

// A server that waits on UDP inputs and sends messages to a listening server
pub struct UDPTransponder {
    config: Config,
}

impl UDPTransponder {
    pub fn new(config: &Config) -> UDPTransponder {
        let conf = config.clone();
        UDPTransponder {
            config: conf,
        }
    }

    /// Starts receiver and sender threads
    pub fn run(&mut self) -> Result<(), io::Error>{
        let sync_pair = Arc::new((Mutex::new(Vec::new()), Condvar::new()));

        let sp = sync_pair.clone();
        let config = self.config.clone();
        let server = thread::Builder::new()
            .name("input".into())
            .spawn(move || {
                let _ = run_udp_receiver(config, sp);
            })
            .expect("input thread failed");

        let worker_pool = ThreadPool::new(self.config.num_client_threads as usize);
        let http_client = Arc::new(hyper::Client::new());

        for _ in 0..self.config.num_client_threads {
            let conf = self.config.clone();
            let sp = sync_pair.clone();
            let c = http_client.clone();
            worker_pool.execute(move || {
                let _ = run_json_sender_worker(conf, sp, c);
            });
        }

        let _ = server.join();

        Ok(())
    }
}
