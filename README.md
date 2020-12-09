# 🐣 Léger

Léger is a `no-std` library exposing a crypto-wallet for embedded targets.

It is written in Rust with a focus on [Substrate](https://www.substrate.io/) chains.

## ⚠ Under development

At the moment, the wallet is made to work with Kusama runtimes (or [node template](https://github.com/substrate-developer-hub/substrate-node-template/)). It should not be used in production.

The library currently provides:

- Websocket handshake
- RPC calls:
    - block genesis
    - system version
    - chain info
    - runtime info
    - balance transfers (extrinsic)

Not supported (yet):

- Secured TLS connection for HTTPS
- DNS client

## Implementation

In order to use `leger-rs`, you need to make sure to provide a TCP Client implementation that implements the 
[`TcpClientStack`](https://github.com/rust-embedded-community/embedded-nal/tree/v0.2.0) Trait from the 
[`embedded_nal`](https://github.com/rust-embedded-community/embedded-nal) library.

See [examples](examples) for an implementation on a Unix-based OS using `std::net::TcpStream`.

> ⚠ Implementing TcpClientStack with TcpStream is far from ideal. Make sure you have a node running and accessible at 
`127.0.0.1:9944` (or make sure to change the address in [the example](examples/unix.rs)) if you want the wallet 
to create a socket.
