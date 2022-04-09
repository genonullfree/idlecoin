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

The stats are written out to a file `.idlecoin` in the working directory of the server upon exit. On start, `idlecoin` will attempt to open `.idlecoin` and ingest the stats file to allow loading of previous stats. The stats file will currently autosave every 5 minutes.

## Output

```
[012] Wallet 0xd259fac86ad34f98 Coins: 0:3543 Miner Licenses: 5 Total Cps: 280
  [*] Miners:
  [M:0x4768b820 Cps:73     B:0      L:1 ] [M:0x19b1eb90 Cps:70     B:0      L:1 ] [M:0x5af31ad8 Cps:70     B:0      L:1 ] [M:0xdfac45f1 Cps:67     B:0      L:1 ]
[001] Wallet 0x7d3ce1ed74b2c05f Coins: 455111110:4595339597215277250 Miner Licenses: 25 Total Cps: 1493
  [*] Miners:
  [M:0x566369be Cps:124    B:0      L:2 ] [M:0xa7f6aa75 Cps:118.1K B:1920   L:12] [M:0x7b104097 Cps:118    B:0      L:2 ] [M:0x81599875 Cps:118    B:0      L:2 ]
  [M:0x05d175f8 Cps:118    B:0      L:2 ] [M:0x7107c253 Cps:112    B:0      L:2 ] [M:0x932c39cf Cps:112    B:0      L:2 ] [M:0xb21f74cf Cps:112    B:0      L:2 ]
  [M:0x6bb92dd6 Cps:106    B:0      L:2 ] [M:0x7a55897f Cps:106    B:0      L:2 ] [M:0xfbef289d Cps:106    B:0      L:2 ] [M:0xacbf46a9 Cps:100    B:0      L:1 ]
  [M:0x5b0f4b06 Cps:94     B:0      L:1 ] [M:0x7cd638c2 Cps:49     B:0      L:1 ]

Logged in as Wallet: 0x7d3ce1ed74b2c05f Miner: 0x7cd638c2
Events:
 [2022-04-03 09:41:12] Miner 0xa7f6aa75 bought 1920 boost seconds with 15360 idlecoin
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
  [M:0xa7f6aa75 Cps:118.1K B:1920   L:12]
     ^              ^        ^        ^
     |              |        |        Miner level
     |              |        Boost seconds
     |              Miner Coins-Per-Second
     Unique Miner ID
```

Each wallet supports at least 5 miners, with options to purchase more once the wallet has enough to purchase additional licenses.

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

