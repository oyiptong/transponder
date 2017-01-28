extern crate error_type;

use std;
use std::io;
use std::cmp::max;
use std::error::Error as StdError;
use std::process::exit;
use std::net::SocketAddr;

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
    pub mtu: i32,
    pub receiver_url: String,
    pub num_client_threads: u16,
}

pub fn parse_config() -> Result<Config, Error> {
    let matches = clap_app!(transponder =>
        (version: "0.1.0")
        (about: "A server that proxies UDP JSON payloads and forwards to an HTTP endpoint")
        (@arg MTU: -p --packet_size +takes_value "Max packet size in bytes (minimum 1400)")
        (@arg ADDR: -a --addr +takes_value "The IP:PORT the server listens on (default '127.0.0.1:48656')")
        (@arg CLIENT_THREADS: -c --client_threads +takes_value "The number of HTTP client threads (default 1)")
        (@arg RECEIVER_URL: -s --receiver_url +takes_value "URL for receiver endpoint (default http://localhost:55555/v1/tracking/events)")
    ).get_matches();

    let default_http_client_threads = 4;
    let default_packet_size = 1500;

    let addr = matches.value_of("ADDR").unwrap_or("127.0.0.1:48656");
    let receiver_url = matches.value_of("RECEIVER_URL").unwrap_or("http://localhost:55555/v1/tracking/events");
    let num_http_client_threads = match matches.value_of("CLIENT_THREADS") {
        Some(s) => { try!(s.parse()) },
        None => default_http_client_threads,
    };
    let mtu = match matches.value_of("MTU") {
        Some(s) => { max(try!(s.parse()), default_packet_size) },
        None => default_packet_size,
    };

    println!("addr: udp://{}", addr);
    println!("mtu: {} bytes", mtu);
    println!("receiver_url: {}", receiver_url);
    println!("num_http_client_threads: {}", num_http_client_threads);

    Ok(Config {
        addr: try!(addr.parse()),
        mtu: mtu,
        receiver_url: receiver_url.to_string(),
        num_client_threads: num_http_client_threads,
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
