#[macro_use]
extern crate log;
extern crate env_logger;
extern crate transponder;

use transponder::utils::{
    unexpected_error,
    unexpected_io_error,
    parse_config,
};
use transponder::net::UDPServer;


fn main() {
    env_logger::init().expect("Unable to init logger");

    let config = parse_config().unwrap_or_else({ |e| unexpected_error(e) });

    let udp_server = &mut UDPServer::new(&config);
    udp_server.run().unwrap_or_else({ |e| unexpected_io_error(e) });
}
