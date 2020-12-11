# 🐣 Léger

Léger is a `no-std` library exposing a crypto-wallet for embedded targets.

It is written in Rust with a focus on [Substrate](https://www.substrate.io/) chains.

## ⚠ Under development

At the moment, the wallet is made to work with Kusama runtimes (or [Substrate node template](https://github.com/substrate-developer-hub/substrate-node-template/)). It should not be used in production.

The library currently provides:

- Websocket layer
- RPC calls to:
    - get block genesis
    - get system version
    - get chain info
    - get runtime info
    - send money (through extrinsic)
  
More features are coming, please check the [Issues](https://github.com/fouge/leger-rs/issues).

🙏 Pull Requests are welcome!

## 🏗 Implementation

### TCP stack

In order to use `leger-rs`, you need to make sure to provide a TCP Client implementation that implements the 
[`TcpClient`](https://github.com/rust-embedded-community/embedded-nal/tree/v0.2.0) trait from the 
[`embedded_nal`](https://github.com/rust-embedded-community/embedded-nal) library.

### Key management and signing

Key management must be done safely and signatures should be computed efficiently. 

It is advised to isolate these jobs in a secure element or any secure context. It is left to the user to implement the 
signing-related functions using the `LegerSigner` trait.

Read the Unix example for more info (see below).

### Unix example

See [the Unix example](examples/unix.rs) for an implementation on a Unix-based OS using `std::net::TcpStream`.
