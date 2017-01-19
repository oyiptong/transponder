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


struct UDPTransponder {
    socket: UdpSocket,
    buf: Vec<u8>,
    incoming: Option<(usize, SocketAddr)>,
    sync: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>,
}

impl Future for UDPTransponder {
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
                cvar.notify_one();
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

fn run_server(config: Config, sync: Arc<(Mutex<Vec<Vec<u8>>>, Condvar)>) -> Result<(), Error> {
    let mut ioloop = try!(Core::new());
    let handle = ioloop.handle();
    let sock = try!(UdpBuilder::new_v4());
    try!(sock.reuse_address(true));
    try!(sock.reuse_port(true));
    let listener = try!(sock.bind(&config.addr));
    let listener = try!(UdpSocket::from_socket(listener, &handle));
    let server = UDPTransponder {
        socket: listener,
        buf: vec![0; 1024],
        incoming: None,
        sync: sync,
    };
    try!(ioloop.run(server));
    Ok(())
}

pub struct UDPServer {
    config: Config,
}

impl UDPServer {
    pub fn new(config: &Config) -> UDPServer {
        let conf = config.clone();
        UDPServer {
            config: conf,
        }
    }

    pub fn run(&mut self) -> Result<(), io::Error>{
        let pair = Arc::new((Mutex::new(Vec::new()), Condvar::new()));

        let p = pair.clone();
        let config = self.config.clone();
        let server = thread::Builder::new()
            .name("input".into())
            .spawn(move || {
                let _ = run_server(config, p);
            })
            .expect("input thread failed");

        let worker_pool = ThreadPool::new(self.config.num_client_threads as usize);
        let http_client = Arc::new(hyper::Client::new());

        for _ in 0..self.config.num_client_threads {
            let conf = self.config.clone();
            let sp = pair.clone();
            let c = http_client.clone();
            worker_pool.execute(move || {
                let _ = run_json_sender_worker(conf, sp, c);
            });
        }

        let _ = server.join();

        Ok(())
    }
}
