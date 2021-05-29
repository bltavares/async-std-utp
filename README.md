# async-std-utp

<a href="https://github.com/bltavares/async-std-utp/actions?query=workflow%3AQuickstart+branch%3Amaster">
    <img src="https://img.shields.io/github/workflow/status/bltavares/async-std-utp/Quickstart/master?label=main%20ci" />
</a>
<a href="https://github.com/bltavares/async-std-utp/actions?query=workflow%3ACross-compile+branch%3Amaster">
    <img src="https://img.shields.io/github/workflow/status/bltavares/async-std-utp/Cross-compile/master?label=cross%20ci" />
</a>
<a href="https://crates.io/crates/async-std-utp">
    <img src="https://img.shields.io/crates/v/async-std-utp.svg" />
</a>
<a href="https://docs.rs/async-std-utp">
    <img src="https://docs.rs/async-std-utp/badge.svg" />
</a>

A [Micro Transport Protocol](http://www.bittorrent.org/beps/bep_0029.html)
library implemented in Rust - integrated with [async-std](https://async.rs).

## Overview

The Micro Transport Protocol is a reliable transport protocol built over
UDP. Its congestion control algorithm is
[LEDBAT](http://tools.ietf.org/html/rfc6817), which tries to use as much unused
bandwidth as it can but readily yields to competing flows, making it useful for
bulk transfers without introducing congestion in the network.

The current implementation is somewhat incomplete, lacking a complete implementation of congestion
control. However, it does support packet loss detection (except by timeout) the
Selective Acknowledgment extension, handles unordered and duplicate packets and
presents a stream interface (`UtpStream`).

## Usage

To use `async-std-utp`, add this to your `Cargo.toml`:

```toml
[dependencies]
async-std-utp = "*"
```

## Examples

The simplest example program would be:

```rust

use async_std_utp::UtpStream;
use async_std::prelude::*;

#[async_std::main]
async fn main() {
    // Connect to an hypothetical local server running on port 8080
    let addr = "127.0.0.1:8080";
    let mut stream = UtpStream::connect(addr).await.expect("Error connecting to remote peer");

    // Send a string
    stream.write("Hi there!".as_bytes()).await.expect("Write failed");

    // Close the stream
    stream.close().await.expect("Error closing connection");
}
```

Check out the files under the "examples" directory for more example programs, or run them with `cargo run --example <example_name>`.

## Roadmap

- [x] congestion control
- [x] proper connection closing
  - [x] handle both RST and FIN
  - [x] send FIN on close
  - [x] automatically send FIN on `drop` if not already closed
- [x] sending RST on mismatch
- [x] setters and getters that hide header field endianness conversion
- [x] SACK extension
- [x] handle packet loss
  - [x] send triple-ACK to re-request lost packet (fast resend request)
  - [x] rewind send window and resend in reply to triple-ACK (fast resend)
  - [x] resend packet on ACK timeout
- [x] stream interface
- [x] handle unordered packets
- [x] duplicate packet handling
- [x] listener abstraction
- [x] incoming connections iterator
- [x] time out connection after too many retransmissions
- [ ] path MTU discovery

## License

This library is distributed under similar terms to Rust: dual licensed under the MIT license and the Apache license (version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.

## Considerations

Based on the work of [rust-utp](https://github.com/meqif/rust-utp). Thank you.
