# Tokio OpenSSL TLS Close Hangs

This repo is a minimal reproduction in a bug observed in the project
[tokio-openssl](https://docs.rs/tokio-openssl/latest/tokio_openssl/).

The issue is that if a peer disconnects w/o terminating the handshake, as in the unexpected loss of
network access the other side of the connection will yield forever instead of receiving an empty buffer
indicating a close has occurred.
