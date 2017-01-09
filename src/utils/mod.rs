extern crate threadpool;
extern crate clap;
extern crate error_type;

use std;
use std::io;
use std::error::Error as StdError;
use std::process::exit;
use std::net::SocketAddr;
use self::threadpool::ThreadPool;
use self::clap::App;

pub fn unexpected_error<T>(err :T) -> !
    where T: StdError,
{
    println!("Failure: {}", err.description().to_string());
    exit(1);
}

pub fn unexpected_io_error(err :io::Error) -> ! {
    println!("Failure: {}", err.description().to_string());
    match err.raw_os_error() {
        Some(code) => exit(code),
        None => exit(1),
    }
}

#[derive(Clone)]
pub struct Config {
    pub addr: SocketAddr,
    http_threadpool: ThreadPool,
    num_server_threads: u16,
}

pub fn parse_config() -> Result<Config, Error> {
    let matches = App::new("transponder")
        .version("0.1.0")
        .about("A server that proxies UDP JSON payloads and forwards to SATCOM")
        .args_from_usage(
            "-a --addr=[ADDR] 'The IP:PORT the server listens on (default \"127.0.0.1:48656\")'
             -c --client-threads=[CLIENT-THREADS] 'The number of HTTP client threads (default 1)'
             -t --threads=[THREADS] 'The number of server threads (default 4)'"
        )
        .get_matches();

    let default_server_threads = 4;
    let default_http_client_threads = 1;

    let addr = matches.value_of("ADDR").unwrap_or("127.0.0.1:48656");
    let num_server_threads = match matches.value_of("THREADS") {
        Some(s) => { try!(s.parse()) },
        None => default_server_threads,
    };
    let num_http_client_threads = match matches.value_of("CLIENT-THREADS") {
        Some(s) => { try!(s.parse()) },
        None => default_http_client_threads,
    };

    println!("addr: udp://{}", addr);
    println!("http client threads: {}", num_http_client_threads);
    println!("server threads: {}", num_server_threads);

    Ok(Config {
        addr: try!(addr.parse()),
        http_threadpool: ThreadPool::new(num_http_client_threads),
        num_server_threads: num_server_threads,
    })
}

error_type! {
    #[derive(Debug)]
    pub enum Error {
        Io(io::Error) { },
        AddrParse(std::net::AddrParseError) { },
        Std(Box<StdError + Send + Sync>) {
            desc (e) e.description();
        },
        ParseInt(std::num::ParseIntError) { },
    }
}
