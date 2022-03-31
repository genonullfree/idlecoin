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
struct Wallet {
    id: u64,         // wallet address ID
    supercoin: u64,  // supercoin
    idlecoin: u64,   // idlecoin
    max_miners: u64, // max number of miners
}

#[derive(Copy, Clone, Debug)]
struct Miner {
    miner_id: u32,  // miner address ID
    wallet_id: u64, // wallet address ID
    level: u64,     // current level
    cps: u64,       // coin-per-second
    inc: u64,       // Incrementor value
    pow: u64,       // Next level up value
}

#[derive(Debug)]
struct Connection {
    miner: Miner,         // Miner for connection
    stream: TcpStream,    // TCP connection
    updates: Vec<String>, // Additional info for specific users
}

fn main() -> Result<(), Error> {
    // Create global array of user wallets
    let wallets = Arc::new(Mutex::new(Vec::<Wallet>::new()));

    // Create global array of user connections
    let connections = Arc::new(Mutex::new(Vec::<Connection>::new()));

    // Load previous stats file
    load_stats(&wallets)?;

    // Bind network listener to port
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT)))?;

    let mut signals = Signals::new(&[SIGINT]).unwrap();
    let wallets_save = Arc::clone(&wallets);
    thread::spawn(move || {
        for sig in signals.forever() {
            if sig == SIGINT {
                // Save the current stats file
                save_stats(wallets_save);
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
                        s.write_all(format!("\n{}\n", e).as_bytes())
                            .expect("Failed to send");
                    }
                    s.shutdown(Shutdown::Both).expect("Failed to shutdown");
                    continue;
                }
            };

            // Set read timeout
            s.set_read_timeout(Some(Duration::new(0, 1)))
                .expect("Unable to set read tiemout");

            let updates = vec![format!("\nLogged in as: 0x{:016x}\n\nAvailable commands:\n'c'<enter>\tPurchase Cps with idlecoin\n'm'<enter>\tPurchase a new miner license\n\nCommand:\n", miner.wallet_id).to_owned()];
            let conn = Connection {
                miner,
                stream: s,
                updates,
            };

            let mut c = connections_close.lock().unwrap();
            c.push(conn);
            drop(c);
        }
    });

    // Main loop
    let mut action_updates = Vec::<String>::new();
    loop {
        read_inputs(&connections, &wallets, &mut action_updates);

        // Calculate miner performance and update stats
        process_miners(&connections, &wallets);

        // Send updates to all connected miners
        let mut msg = print_wallets(&connections, &wallets);

        // Roll dice for random actions
        action_miners(&connections, &wallets, &mut action_updates);

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
    let _ = stream.read(&mut id_raw[..])?;

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
    })
}

fn read_inputs(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
    msg: &mut Vec<String>,
) {
    let mut cons = connections.lock().unwrap();

    for c in cons.iter_mut() {
        let mut buf = [0; 1024];
        let len = match c.stream.read(&mut buf) {
            Ok(l) => l,
            Err(_) => continue,
        };

        if len > 0 {
            println!("read {:?} {:?}", c.stream, &buf[..len]);
            if buf.contains(&99) {
                // 'b'
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        if w.idlecoin < 1024 {
                            c.updates.push(
                                "You need at least 1024 idlecoin to be able to purchase Cps\n"
                                    .to_string(),
                            );
                            continue;
                        }
                        let v = u64::BITS - w.idlecoin.leading_zeros() - 1;
                        let cost = 1u64.checked_shl(v).unwrap_or(0);
                        let cps = if cost > 0 {
                            1u64.checked_shl(v / 2).unwrap_or(0)
                        } else {
                            0
                        };

                        let t: DateTime<Local> = Local::now();
                        msg.insert(
                            0,
                            format!(
                                "    [{}] Miner 0x{:08x} bought {} Cps with {} idlecoin\n",
                                t, c.miner.miner_id, cps, cost
                            ),
                        );
                        sub_idlecoins(w, cost);
                        c.miner.cps = c.miner.cps.saturating_add(cps);
                    }
                }
                drop(wals);
            }
            if buf.contains(&109) {
                // 'w'
                let mut wals = wallets.lock().unwrap();
                for w in wals.iter_mut() {
                    if w.id == c.miner.wallet_id {
                        if w.max_miners >= 10 {
                            c.updates
                                .push("You cannot purchase any more miners\n".to_string());
                            continue;
                        }
                        let cost = u64::MAX / (100000 >> (w.max_miners - 5));
                        if w.idlecoin > cost {
                            let t: DateTime<Local> = Local::now();
                            msg.insert(0, format!("    [{}] Wallet 0x{:016x} bought a new miner license with {} idlecoin\n", t, c.miner.wallet_id, cost));
                            sub_idlecoins(w, cost);
                            w.max_miners += 1;
                        } else {
                            c.updates.push(format!(
                                "You need {} idlecoin to purchase another miner license\n",
                                cost
                            ));
                        }
                    }
                }
                drop(wals);
            }
        }
    }
    drop(cons);
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
        let wal = &format!(
            "[{:03}] Wallet 0x{:016x} Coins: {}:{}\n",
            gens.len() - i,
            g.id,
            g.supercoin,
            g.idlecoin,
        )
        .to_owned();
        let mut min: String = "".to_string();
        for c in cons.iter() {
            if c.miner.wallet_id == g.id {
                min += &format!(
                    "    [+] Miner 0x{:08x} Cps: {} Level: {} Inc: {} Pow: {}\n",
                    c.miner.miner_id, c.miner.cps, c.miner.level, c.miner.inc, c.miner.pow,
                )
                .to_owned();
            }
        }
        if !min.is_empty() {
            msg += wal;
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

fn action_miners(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
    msg: &mut Vec<String>,
) {
    let mut rng = rand::thread_rng();

    let mut cons = connections.lock().unwrap();

    for c in cons.iter_mut() {
        let t: DateTime<Local> = Local::now();
        let x: u64 = rng.gen();

        if x % 15552000 == 0 {
            // 0.00000006430041152263 % chance
            let mut wal = wallets.lock().unwrap();
            for w in wal.iter_mut() {
                if w.id == c.miner.wallet_id {
                    w.supercoin -= w.supercoin.saturating_div(10);
                    let coins = w.idlecoin.saturating_div(10);
                    sub_idlecoins(w, coins);
                    msg.insert(
                        0,
                        format!(
                            " [{}] Wallet 0x{:016x} was taxed 10% by the IRS!\n",
                            t, c.miner.miner_id
                        ),
                    );
                }
            }
            drop(wal);
        } else if x % 10000 == 0 {
            // 0.01 % chance
            let level = c.miner.level;
            dec_level(&mut c.miner);
            if level != c.miner.level {
                msg.insert(
                    0,
                    format!(" [{}] Miner 0x{:08x} lost a level\n", t, c.miner.miner_id),
                );
            }
        } else if x % 10000 <= 2 {
            // 0.02 % chance
            let level = c.miner.level;
            inc_level(&mut c.miner);
            if level != c.miner.level {
                msg.insert(
                    0,
                    format!(" [{}] Miner 0x{:08x} leveled up\n", t, c.miner.miner_id),
                );
            };
        } else if x % 10000 <= 3 {
            // .01 % chance
            c.miner.cps += c.miner.cps.saturating_div(10);
            msg.insert(
                0,
                format!(
                    " [{}] Miner 0x{:08x} gained 10% CPS boost\n",
                    t, c.miner.miner_id
                ),
            );
        }
    }

    if msg.len() > 5 {
        msg.resize(5, "".to_owned());
    };

    drop(cons);
}

fn process_miners(connections: &Arc<Mutex<Vec<Connection>>>, wallets: &Arc<Mutex<Vec<Wallet>>>) {
    let mut cons = connections.lock().unwrap();
    let mut wals = wallets.lock().unwrap();

    for c in cons.iter_mut() {
        // Update miner
        miner_session(&mut c.miner);
        // Update appropriate wallet
        for w in wals.iter_mut() {
            if c.miner.wallet_id == w.id {
                add_idlecoins(w, c.miner.cps);
            }
        }
    }
    drop(wals);
    drop(cons);
}

fn inc_level(miner: &mut Miner) {
    miner.level = miner.level.saturating_add(1);

    miner.inc = miner.inc.saturating_add(miner.level);

    miner.pow = miner.pow.saturating_mul(10);
}

fn dec_level(miner: &mut Miner) {
    miner.level = miner.level.saturating_sub(1);

    miner.inc = miner.inc.saturating_sub(miner.level);

    miner.pow = miner.pow.saturating_div(10);
}

fn add_idlecoins(mut wallet: &mut Wallet, new: u64) {
    wallet.idlecoin = match wallet.idlecoin.checked_add(new) {
        Some(c) => c,
        None => {
            wallet.supercoin = wallet.supercoin.saturating_add(1);
            let x: u128 = (u128::from(wallet.idlecoin) + u128::from(new)) % u128::from(u64::MAX);
            x as u64
        }
    };
}

fn sub_idlecoins(mut wallet: &mut Wallet, less: u64) {
    wallet.idlecoin = match wallet.idlecoin.checked_sub(less) {
        Some(c) => c,
        None => {
            if wallet.supercoin > 0 {
                wallet.supercoin = wallet.supercoin.saturating_sub(1);
                (u128::from(less) - u128::from(wallet.idlecoin) - u128::from(u64::MAX))
                    .try_into()
                    .unwrap()
            } else {
                0
            }
        }
    };
}

fn miner_session(mut miner: &mut Miner) {
    // Level up
    if miner.cps >= miner.pow {
        inc_level(miner);
    }

    // Increment cps
    miner.cps = miner.cps.saturating_add(miner.inc + miner.level);
}

fn load_stats(wallets: &Arc<Mutex<Vec<Wallet>>>) -> Result<(), Error> {
    let mut j = String::new();

    // Attempt to open and read the saved stats file
    let mut file = match File::open(&SAVE) {
        Ok(f) => f,
        Err(_) => {
            println!("No stats file found.");
            return Ok(());
        }
    };

    file.read_to_string(&mut j)?;

    // Exit if file is empty
    if j.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "No data to load"));
    }

    // Attempt to deserialize the json file data
    println!("Loading stats...");
    if let Ok(mut wallet) = serde_json::from_str(&j) {
        // Update the wallets struct
        let mut gens = wallets.lock().unwrap();
        gens.append(&mut wallet);
        drop(gens);
        println!("Successfully loaded stats file {}", SAVE);
    } else {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Failed to load {}", SAVE),
        ));
    }

    Ok(())
}

fn save_stats(wallets: Arc<Mutex<Vec<Wallet>>>) {
    // Serialize the stats data to json
    println!("Saving stats...");
    let gens = wallets.lock().unwrap();
    let j = serde_json::to_string_pretty(&gens.deref()).unwrap();
    drop(gens);

    // Open the stats file for writing
    let mut file = match File::create(&SAVE) {
        Ok(f) => f,
        Err(_) => {
            println!("Error opening {} for writing!", SAVE);
            return;
        }
    };

    // Write out the json stats data
    let len = file.write(j.as_bytes()).unwrap();
    if j.len() != len {
        println!("Error writing save data to {}", SAVE);
        return;
    }

    println!("Successfully saved data to {}", SAVE);
}
