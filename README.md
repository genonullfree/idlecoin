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
[004] Wallet 0xe7299d9f952a6c31 Coins: 0:16324
    [+] Miner 0x91e5e95a Cps: 603 Level: 2
[003] Wallet 0xe8b26b26f8b447f9 Coins: 0:58426
    [+] Miner 0xedadbfe7 Cps: 342 Level: 2
[001] Wallet 0x7d3ce1ed74b2c05f Coins: 0:261941
    [+] Miner 0x3b7c1ffb Cps: 430 Level: 2
    [+] Miner 0x2c37118e Cps: 546 Level: 3

Events:
 [!] Miner 0x91e5e95a gained 10% CPS boost
 [!] Miner 0x2c37118e leveled up
 [!] Miner 0xedadbfe7 gained 10% CPS boost
 [!] Miner 0x2c37118e gained 10% CPS boost
 [!] Miner 0x2c37118e gained 10% CPS boost

Logged in as: 0x7d3ce1ed74b2c05f%       
```

The display is updated every second. Only Wallets with active miners will be displayed, but the rank number will be accurate for all Wallets on the server.

### Wallet

```
[002] Wallet 0xe8b26b26f8b447f9 Coins: 0:58426
^            ^                         ^
|            |                         Supercoins:Idlecoins
|            Unique Wallet ID
Rank number
```

### Miners

```
    [+] Miner 0x2c37118e Cps: 546 Level: 3
              ^               ^          ^
              |               |          Miner level
              |               Miner Coins-Per-Second
              Unique Miner ID
```

Each wallet supports at least 5 miners.

### Events

```
Events:
 [!] Miner 0x91e5e95a gained 10% CPS boost
 [!] Miner 0x2c37118e leveled up
 [!] Miner 0xedadbfe7 gained 10% CPS boost
 [!] Miner 0x2c37118e gained 10% CPS boost
 [!] Miner 0x2c37118e gained 10% CPS boost
```

These events are newest on top, and only the most recent 5 are displayed.

These events are special random events that can happen:

1. Gain 10% CPS -- 0.01% chance
1. Gain 1 Level -- 0.02% chance
1. Lose 1 Level -- 0.01% chance


## Auto-Login

To setup your terminal to auto-login in case the connection with the server is interrupted (say, because the alpha software crashed or the server is being upgraded to a new version) you can use the following bash command, substituting your info for `<USER>`, `<SERVER>`, and `<PORT>`:
```bash
while true; do echo <USER> |nc <SERVER> <PORT>; done
```

*NOTE*: When logging in this way you will be unable to purchase upgrades with this miner. STDIN is not routed through `nc`.

When that happens the miner level and CPS will be reset to 0, just like starting a new connection.

