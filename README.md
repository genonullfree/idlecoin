# IDLECOIN

This is an idle game where the point is to open a netcat connection to the server and keep it open as long as possible. The longer the connection is active, the more powerful the `idlecoin` generator becomes.

## Use

To start, run the `idlecoin` server:
```rust
cargo run
```

To join a server, use `netcat` or `telnet` to connect to the server:
```bash
# netcat <ip-of-server> <port: default 7654>
nc 127.0.0.1 7654
# or
telnet localhost 7654
```
