use std::io;
use std::process::exit;
use std::error::Error;

pub fn unexpected_error<T>(err :T) -> !
    where T: Error,
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
