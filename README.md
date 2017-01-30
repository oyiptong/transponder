# transponder

`transponder` is a utility that receives payloads that have to be non-blocking and forwards them to
endpoints with less strict response time requirements.

It is important to note that `transponder` only relays data received verbatim.

The expectation is that the recipient would know how to persist the message if need be and/or
transmit to a non-local location.

The only supported emitted payload for now is `JSON` over HTTP. The input data is received and
assumed to be valid `JSON`.

Both Mac OS and Linux are supported.

# guarantees

* Messages are sent approximately in the order they are received, but there are no guarantees
* There is natural queuing of upload jobs, but it is not-persistent

# implementation

Input is single-threaded but evented, output is managed through a threadpool.
Using the threadpool setting is a good way to limit concurrency to the recipient endpoint.

By default, `transponder` listens on `127.0.0.1:48656`. The receiver thread accepts datagram inputs
from the socket, produces input into a queue.

A set of worker threads pick up jobs from the queue and transmit the raw payloads to a designated
receipient URL over HTTP. By default, this is: http://localhost:55555/v1/tracking/events


# options

`transponder` options can be obtained from the command line by typing the `--help` parameter.


```
$ transponder -h
```

An overview of the options available:

| -a, --addr <ADDR> | The IP:PORT the server listens on (default '127.0.0.1:48656') |
| -c, --client_threads <CLIENT_THREADS> | The number of HTTP client threads (default 1) |
| -p, --packet_size <MTU> | Max packet size in bytes (default/minimum 1400) |
| -s, --receiver_url <RECEIVER_URL> | URL for receiver endpoint (default http://localhost:55555/v1/tracking/events) |

## Build instructions

Requirements are:
* Mac OS or Linux (BSD's may be supported, but are untested)
* Rust 1.13.0 or 1.14.0

`rustup` is recommended for compiler management.

To install dependencies and build with compiler optimizations, simply run:

```
$ cargo build --release
```

The statically compiled binary will be found at:

```
./target/release/transponder
```
