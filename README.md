```
 /$$       /$$ /$$                               /$$
|__/      | $$| $$                              |__/
 /$$  /$$$$$$$| $$  /$$$$$$   /$$$$$$$  /$$$$$$  /$$ /$$$$$$$
| $$ /$$__  $$| $$ /$$__  $$ /$$_____/ /$$__  $$| $$| $$__  $$
| $$| $$  | $$| $$| $$$$$$$$| $$      | $$  \ $$| $$| $$  \ $$
| $$| $$  | $$| $$| $$_____/| $$      | $$  | $$| $$| $$  | $$
| $$|  $$$$$$$| $$|  $$$$$$$|  $$$$$$$|  $$$$$$/| $$| $$  | $$
|__/ \_______/|__/ \_______/ \_______/ \______/ |__/|__/  |__/
```

This is an idle game where the point is to open a netcat connection to the server and keep it open as long as possible. The longer the connection is active, the more powerful the `idlecoin` miners become.

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
[004] Wallet 0x5bd33994fa398d38 Coins: 0:90077 Miner Licenses: 5 Total Cps: 952
  [*] Miners:
          0xa0cd3980
             952 Cps
              0B  2L
[002] Wallet 0xc5bef0bd52e469b7 Coins: 0:4969441343 Miner Licenses: 5 Total Cps: 2742
  [*] Miners:
          0x3e83596c           0xbeb5504f           0x97d8c768
             916 Cps              916 Cps              910 Cps
              0B  2L               0B  2L               0B  2L
[001] Wallet 0x7d3ce1ed74b2c05f Coins: 455111110:4595128692089938046 Miner Licenses: 25 Total Cps: 7730
  [*] Miners:
          0x876be1c6           0xcca76078           0x64031bd5           0x50867202           0xe2c5a51f
            1260 Cps             1110 Cps             1100 Cps             1090 Cps             1090 Cps
           1914B  3L               0B  3L               0B  3L               0B  3L               0B  3L
          0x5afa7dd7           0x2917f60b
            1070 Cps             1010 Cps
              0B  3L               0B  3L

Events:
 [2022-04-03 09:41:12] Miner 0x876be1c6 bought 1920 boost seconds with 15360 idlecoin

Logged in as Wallet: 0x7d3ce1ed74b2c05f Miner: 0x876be1c6
Commands:
'b'<enter>      Purchase 128 boost for 1024 idlecoin
```

The display is updated every second. Only Wallets with active miners will be displayed, but the rank number will be accurate for all Wallets on the server.

### Wallet

```
[002] Wallet 0xc5bef0bd52e469b7 Coins: 0:4969441343 Miner Licenses: 5 Total Cps: 2742
^            ^                         ^                            ^            ^
|            |                         |                            |            Total amount of Cps for all Miners
|            |                         |                            Max Number of Miners for Wallet
|            |                         Supercoins:Idlecoins
|            Unique Wallet ID
Rank number
```

### Miners

```
  [*] Miners:
          0x876be1c6    <- Unique Miner ID
            1260 Cps    <- Miner Coins-Per-Second
           1914B  3L    <- Miner level
           ^
           Boost seconds
```

Each wallet supports at least 5 miners.

### Events

```
Events:
 [2022-04-03 09:41:12] Miner 0x876be1c6 bought 1920 boost seconds with 15360 idlecoin
```

These events are newest on top, and only the most recent 5 are displayed.

These events can be a mix of users purchasing upgrades for their miners or special random events that can happen:

1. Gain 10% CPS -- 0.01% chance
1. Gain 1 Level -- 0.02% chance
1. Lose 1 Level -- 0.01% chance
1. IRS auditing -- 0.00000006430041152263% chance


## Auto-Login

To setup your terminal to auto-login in case the connection with the server is interrupted (say, because the alpha software crashed or the server is being upgraded to a new version) you can use the following bash command, substituting your info for `<USER>`, `<SERVER>`, and `<PORT>`:
```bash
while true; do echo <USER> |nc <SERVER> <PORT>; done
```

*NOTE*: When logging in this way you will be unable to purchase upgrades with this miner. STDIN is not routed through `nc`.

When that happens the miner level and CPS will be reset to 0, just like starting a new connection.

