```
 /$$       /$$ /$$                                  /$$
|__/      | $$| $$                                 |__/
 /$$  /$$$$$$$| $$  /$$$$$$   /$$$$$$$  /$$$$$$     /$$ /$$$$$$$
| $$ /$$__  $$| $$ /$$__  $$ /$$_____/ /$$__  $$   | $$| $$__  $$
| $$| $$  | $$| $$| $$$$$$$$| $$      | $$  \ $$   | $$| $$  \ $$
| $$| $$  | $$| $$| $$_____/| $$      | $$  | $$   | $$| $$  | $$
| $$|  $$$$$$$| $$|  $$$$$$$|  $$$$$$$|  $$$$$$//$$| $$| $$  | $$
|__/ \_______/|__/ \_______/ \_______/ \______/|__/|__/|__/  |__/
```

This is an idle game where the point is to open a netcat connection to the server and keep it open as long as possible. The longer the connection is active, the more powerful the `idlecoin` miners become.

The game has evolved to become semi-interactive, as there is now an element of purchasing upgrades to improve the performance of your miners, but this is entirely optional.

## Use

To start, run the `idlecoin` server:
```rust
cargo run --release
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
[007] Wallet 0x9d75d7d276240c38 Miner Licenses: 5 Chronocoin: 8850 Randocoin: 208 Coins: 0:24436823427358 Total Cps: 10
  [*] Miners:
  [M:0xf56f7348 Cps:2  B:0  L:0 ] [M:0x75bcdd68 Cps:2  B:0  L:0 ] [M:0x52fb5cfa Cps:2  B:0  L:0 ] [M:0x872c9c90 Cps:2  B:0  L:0 ]
  [M:0xd8d97efb Cps:2  B:0  L:0 ]

[003] Wallet 0x7d3ce1ed74b2c05f Miner Licenses: 12 Chronocoin: 827 Randocoin: 128 Coins: 23020:18440315426171777220 Total Cps: 6116500
  [*] Miners:
  [M:0x6326b8e9 Cps:3.6M B:39.9K L:7 ] [M:0xdfd14fd0 Cps:573.0K B:57.7K L:5 ] [M:0x5448a361 Cps:621.6K B:57.7K L:6 ] [M:0x91d40505 Cps:139.4K B:0  L:5 ]
  [M:0xe0a14782 Cps:153.8K B:0  L:5 ] [M:0x89fe5e31 Cps:182.4K B:0  L:5 ] [M:0x4a836f86 Cps:140.8K B:0  L:5 ] [M:0xd1875fb2 Cps:170.6K B:0  L:5 ]
  [M:0xfa610bee Cps:160.3K B:0  L:7 ] [M:0xf961bd80 Cps:181.6K B:0  L:5 ] [M:0x992601a8 Cps:198.5K B:0  L:7 ]

[002] Wallet 0xa1e373bb74ac15d4 Miner Licenses: 10 Chronocoin: 8854 Randocoin: 128 Coins: 76095:6748642428673042 Total Cps: 1561221
  [*] Miners:
  [M:0x08ba1c7f Cps:200.4K B:0  L:6 ] [M:0xfc73221a Cps:147.2K B:0  L:6 ] [M:0x8a1af29b Cps:166.1K B:0  L:5 ] [M:0xecec3887 Cps:144.9K B:0  L:5 ]
  [M:0x9f472d0d Cps:147.7K B:0  L:5 ] [M:0x33adde6a Cps:288.3K B:0  L:8 ] [M:0x661a3b06 Cps:161.9K B:0  L:6 ] [M:0x1de985eb Cps:147.9K B:0  L:5 ]


Events:
 [2022-04-10 22:01:24] Miner 0xf56f7348 leveled up
 [2022-04-10 21:59:21] Miner 0xdb749837 leveled up
 [2022-04-10 21:56:17] Miner 0xf3c8e016 gained 10% CPS boost
 [2022-04-10 21:53:48] Miner 0xfa610bee leveled up
 [2022-04-10 21:51:54] Miner 0x7d3ce1ed74b2c05f travelled 7 hours forward in time with 7000 chronocoins

Logged in as Wallet: 0x7d3ce1ed74b2c05f Miner: 0x6326b8e9
Commands:
'b'<enter>      Purchase 128 seconds of Miner Boost for 2097152 idlecoin
```

The display is updated every second. Only Wallets with active miners will be displayed, but the rank number will be accurate for all Wallets on the server.

### Wallet

```
[002] Wallet 0xa1e373bb74ac15d4 Miner Licenses: 10 Chronocoin: 8854 Randocoin: 128 Coins: 76095:6748642428673042 Total Cps: 1561221
^            ^                                  ^              ^               ^          ^                                 ^
|            |                                  |              |               |          |                                 Total amount of Cps for all Miners
|            |                                  |              |               |          Number of Supercoins:Idlecoins in Wallet
|            |                                  |              |               Number of Randocoins in Wallet
|            |                                  |              Number of Chronocoins in Wallet
|            |                                  Max number of Miners for Wallet
|            Unique Wallet ID
Rank number
```

### Miners

```
  [*] Miners:
  [M:0x23e9027d Cps:156.9K B:0  L:6 ]
     ^              ^        ^    ^
     |              |        |    Miner level
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

### Purchasing Upgrades

Now certain upgrades can be purchased while playing! Idlecoin will notify you when something is available for purchase and how much it will cost. You can bundle purchasing many items into 1 command string by entering multiple letters before hitting `enter`. Here are the current upgrades available and their associated purchase commands:

* *Boost*
> Boost will allow a miner to increase it's Cps 3x faster with the command letter `b`. Boost is purchased in bundles of 128 seconds and costs log(2) of your Miners current Cps value. As your Miner becomes more powerful and generates more Cps, its cost for Boost will increase.

* *Miner License*
> Purchasing additional Miner Licenses with the command letter `m` will allow your Wallet to connect more concurrent Miners. The price for the Miner Licenses increases for each additional License according to this function: `u64::MAX / (0x100000 >> (max_miners - 5))`. `max_miners` is a per-Wallet value and starts out at `5` and can currently grow to 12 through additional Licenses.

* *Time Travel*
> Time travel can be purchased via Chronocoins with the command letter `c`. These coins increase monatomically, 1 per second, for every Wallet that has at least 1 Miner currently attached to it. Time travel costs 1000 Chronocoins and will allwo the current Miner to travel forward in time by 1 hour. This time travel will cause the Miner to accumulate all Cps and Idlecoins as if the hour had actually passed.

Commands can be chained together and will be executed sequentially. For example, entering a command buffer of:
```
bbbmcc
```
Will cause the current Miner to purchase 384 Boost-seconds (128*3), an additional Miner License, and then time travel forward by 2 hours, assuming all of the necessary funds are available. A command buffer with insufficient funds will continue to be parsed until the entire buffer is read, even if a command fails due to insufficient funds or reaching a maximum value.

## Auto-Login

To setup your terminal to auto-login in case the connection with the server is interrupted (say, because the alpha software crashed or the server is being upgraded to a new version) you can use the following bash command, substituting your info for `<USER>`, `<SERVER>`, and `<PORT>`:
```bash
while true; do echo <USER> |nc <SERVER> <PORT>; done
```

*NOTE*: When logging in this way you will be unable to purchase upgrades with this miner. STDIN is not routed through `nc`.

When that happens the miner level and CPS will be reset to 0, just like starting a new connection.

