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

The stats are written out to a file `.idlecoin` in the working directory of the server upon exit. On start, `idlecoin` will attempt to open `.idlecoin` and ingest the stats file to allow loading of previous stats.

## Output

```
+++
[002] Wallet 0x857933191944b3ba Coins 0:72287, CPS: 3016
[001] Wallet 0x7d3ce1ed74b2c05f Coins 0:569629, CPS: 28, level: 1 <= ***
^            ^                        ^         ^               ^
|            |                        |         |               Mining generation level
|            |                        |         Coins-per-second
|            |                        Supercoins:Idlecoins
|            Wallet ID
Rank number
```

* `+++`: The delimiter between updates
* `***`: The current miner marker
