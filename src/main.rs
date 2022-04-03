use std::fs::File;
use std::hash::Hasher;
use std::io::{Error, ErrorKind};
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use chrono::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use signal_hook::{consts::SIGINT, iterator::Signals};
use xxhash_rust::xxh3;

mod file;
mod commands;
mod miner;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CLR: &str = "\x1b[2J\x1b[;H";
const PORT: u16 = 7654;
const SAVE: &str = ".idlecoin";
const IDLECOIN: &str = r"

 /$$       /$$ /$$                               /$$
|__/      | $$| $$                              |__/
 /$$  /$$$$$$$| $$  /$$$$$$   /$$$$$$$  /$$$$$$  /$$ /$$$$$$$
| $$ /$$__  $$| $$ /$$__  $$ /$$_____/ /$$__  $$| $$| $$__  $$
| $$| $$  | $$| $$| $$$$$$$$| $$      | $$  \ $$| $$| $$  \ $$
| $$| $$  | $$| $$| $$_____/| $$      | $$  | $$| $$| $$  | $$
| $$|  $$$$$$$| $$|  $$$$$$$|  $$$$$$$|  $$$$$$/| $$| $$  | $$
|__/ \_______/|__/ \_______/ \_______/ \______/ |__/|__/  |__/
";
const BANNER: &str = "
Source: https://github.com/genonullfree/idlecoin

Please enter your username: ";

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Wallet {
    id: u64,         // wallet address ID
    supercoin: u64,  // supercoin
    idlecoin: u64,   // idlecoin
    max_miners: u64, // max number of miners
}

#[derive(Copy, Clone, Debug)]
pub struct Miner {
    miner_id: u32,  // miner address ID
    wallet_id: u64, // wallet address ID
    level: u64,     // current level
    cps: u64,       // coin-per-second
    inc: u64,       // Incrementor value
    pow: u64,       // Next level up value
    boost: u64,     // Seconds of boosted cps
}

#[derive(Debug)]
pub struct Connection {
    miner: Miner,         // Miner for connection
    stream: TcpStream,    // TCP connection
    updates: Vec<String>, // Additional info for specific users
    purchases: Vec<String>, // Prices and available things for purchasing
}

fn main() -> Result<(), Error> {
    // Create global array of user wallets
    let wallets = Arc::new(Mutex::new(Vec::<Wallet>::new()));

    // Create global array of user connections
    let connections = Arc::new(Mutex::new(Vec::<Connection>::new()));

    // Load previous stats file
    file::load_stats(&wallets)?;

    // Bind network listener to port
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT)))?;

    let mut signals = Signals::new(&[SIGINT]).unwrap();
    let wallets_save = Arc::clone(&wallets);
    thread::spawn(move || {
        for sig in signals.forever() {
            if sig == SIGINT {
                // Save the current stats file
                file::save_stats(wallets_save);
                std::process::exit(0);
            }
        }
    });

    // Handle connection in new thread
    let connections_close = Arc::clone(&connections);
    let wallets_close = Arc::clone(&wallets);

    thread::spawn(move || {
        // Listen for connections
        for stream in listener.incoming() {
            let mut s = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Allow user session to login
            let miner = match login(&s, &wallets_close, &connections_close) {
                Ok(m) => m,
                Err(e) => {
                    if e.kind() == ErrorKind::ConnectionRefused {
                        match s.write_all(format!("\n{}\n", e).as_bytes()) {
                            Ok(_) => (),
                            Err(e) => println!("Failed to send: {e}"),
                        };
                    }
                    match s.shutdown(Shutdown::Both) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to shutdown: {e}"),
                    };
                    continue;
                }
            };

            // Set read timeout
            match s.set_read_timeout(Some(Duration::new(0, 1))) {
                Ok(_) => (),
                Err(e) => {
                    println!("Unable to set read tiemout: {e}");
                    continue;
                }
            };

            let updates = vec![format!("\nLogged in as: 0x{:016x}\n", miner.wallet_id).to_owned()];
            let conn = Connection {
                miner,
                stream: s,
                updates,
                purchases: vec![],
            };

            let mut c = connections_close.lock().unwrap();
            c.push(conn);
            drop(c);
        }
    });

    // Main loop
    let mut action_updates = Vec::<String>::new();
    loop {
        commands::read_inputs(&connections, &wallets, &mut action_updates);

        // Calculate miner performance and update stats
        miner::process_miners(&connections, &wallets);

        // Send updates to all connected miners
        let mut msg = print_wallets(&connections, &wallets);

        // Roll dice for random actions
        miner::action_miners(&connections, &wallets, &mut action_updates);

        // Format the update messages
        format_msg(&mut msg, &action_updates);

        // Send wallet updates to all connections every 2 seconds
        send_updates_to_all(msg, &connections);

        // Sleep from all that hard work
        sleep(Duration::from_secs(1));
    }
}

fn login(
    mut stream: &TcpStream,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
    connections: &Arc<Mutex<Vec<Connection>>>,
) -> Result<Miner, Error> {
    // Request userid
    let msg = format!("{}Welcome to{}v{}\n\n{}", CLR, IDLECOIN, VERSION, BANNER);
    stream.write_all(msg.as_bytes())?;

    // Read userid
    let mut id_raw: [u8; 1024] = [0; 1024];

    // Only read 0-1023 to have the end NULL so we can safely do the
    // \r\n => \n\0 conversion
    let len = stream.read(&mut id_raw[..1023])?;
    for i in 0..len {
        if id_raw[i] == b'\r' && id_raw[i + 1] == b'\n' {
            id_raw[i] = b'\n';
            id_raw[i + 1] = 0x0;
        }
    }

    // Calculate the hash of the wallet_id
    let mut hash = xxh3::Xxh3::new();
    hash.write(&id_raw);
    let wallet_id = hash.finish();

    // ID or create new wallet
    let mut wals = wallets.lock().unwrap();
    let mut found = false;
    let mut max_miners = 5;
    for w in wals.iter() {
        if w.id == wallet_id {
            found = true;
            max_miners = w.max_miners;
        }
    }
    if !found {
        wals.push(Wallet {
            id: wallet_id,
            supercoin: 0,
            idlecoin: 0,
            max_miners,
        });
    }
    drop(wals);

    // Limit number of connections / miners
    let cons = connections.lock().unwrap();
    let mut num = 0;
    for c in cons.iter() {
        if wallet_id == c.miner.wallet_id {
            num += 1;
        }
    }
    if num >= max_miners {
        let msg = format!(
            "Connection refused: Too many miners connected for user 0x{:016x} (max: {})",
            wallet_id, max_miners,
        );
        println!("{}", msg);
        drop(cons);
        return Err(Error::new(ErrorKind::ConnectionRefused, msg));
    }

    // Generate a unique random miner_id
    let mut rng = rand::thread_rng();
    let mut miner_id: u32;
    'retry: loop {
        miner_id = rng.gen();
        for c in cons.iter() {
            if miner_id == c.miner.miner_id {
                continue 'retry;
            }
        }
        break;
    }
    drop(cons);

    println!(
        "User++ U:0x{:016x} M:0x{:08x} from: {:?}",
        wallet_id, miner_id, stream
    );

    // Create new Miner
    Ok(Miner {
        miner_id,
        wallet_id,
        level: 0,
        cps: 0,
        inc: 1,
        pow: 10,
        boost: 0,
    })
}

fn print_wallets(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
) -> String {
    let mut msg = format!("{}{}v{}\n\n", CLR, IDLECOIN, VERSION);
    let mut gens = wallets.lock().unwrap().deref().clone();
    let cons = connections.lock().unwrap();

    gens.sort_by(|a, b| a.idlecoin.cmp(&b.idlecoin));
    gens.sort_by(|a, b| a.supercoin.cmp(&b.supercoin));

    for (i, g) in gens.iter().enumerate() {
        let mut min = String::new();
        let mut total_cps = 0u128;
        let mut miner_top: Vec<String> = vec![];
        let mut miner_mid: Vec<String> = vec![];
        let mut miner_bot: Vec<String> = vec![];
        let mut num = 0;
        for c in cons.iter() {
            if c.miner.wallet_id == g.id {
                miner_top.push(format!("{:>11}x{:0>8x} ", 0, c.miner.miner_id,).to_owned());
                miner_mid.push(format!("{:>16} Cps ", c.miner.cps,).to_owned());
                miner_bot
                    .push(format!("{:>11}B {:>5}L ", c.miner.boost, c.miner.level,).to_owned());
                total_cps += c.miner.cps as u128;
                num += 1;
                if num > 4 {
                    for i in &miner_top {
                        min += i;
                    }
                    min += "\n";
                    for i in &miner_mid {
                        min += i;
                    }
                    min += "\n";
                    for i in &miner_bot {
                        min += i;
                    }
                    min += "\n";
                    num = 0;
                    miner_top = vec![];
                    miner_mid = vec![];
                    miner_bot = vec![];
                }
            }
        }
        if num > 0 {
            for i in &miner_top {
                min += i;
            }
            min += "\n";
            for i in &miner_mid {
                min += i;
            }
            min += "\n";
            for i in &miner_bot {
                min += i;
            }
            min += "\n";
        }
        let wal = &format!(
            "[{:03}] Wallet 0x{:016x} Coins: {}:{} Miner Licenses: {} Total Cps: {}\n",
            gens.len() - i,
            g.id,
            g.supercoin,
            g.idlecoin,
            g.max_miners,
            total_cps,
        )
        .to_owned();
        if !min.is_empty() {
            msg += wal;
            msg += "  [*] Miners\n";
            msg += &min;
        }
    }
    drop(cons);
    drop(gens);

    msg
}

fn format_msg(input: &mut String, actions: &[String]) {
    if actions.is_empty() {
        return;
    }

    input.push_str(&"\nEvents:\n".to_string());

    for a in actions {
        input.push_str(a);
    }
}

fn send_updates_to_all(input: String, connections: &Arc<Mutex<Vec<Connection>>>) {
    let mut cons = connections.lock().unwrap();
    let mut rem = Vec::<usize>::new();

    for (i, c) in cons.iter_mut().enumerate() {
        // Append updates to input message
        let mut msg = input.clone();
        for u in &c.updates {
            msg.push_str(u);
        }

        // Send message to connection
        if c.stream.write_all(msg.as_bytes()).is_err() {
            // If error, mark miner for disconnection
            rem.push(i);
            println!(
                "User-- U:0x{:016x} M:0x{:08x} from: {:?}",
                c.miner.wallet_id, c.miner.miner_id, c.stream
            );
        }
        while c.updates.len() > 1 {
            c.updates.pop();
        }
    }

    // Remove disconnected miners
    rem.sort_unstable();
    rem.reverse();
    for i in rem.iter() {
        cons.remove(*i);
    }
    drop(cons);
}
