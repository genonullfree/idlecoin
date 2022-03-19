use std::fs::File;
use std::hash::Hasher;
use std::io::{Error, ErrorKind};
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

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
    id: u64,        // wallet address ID
    supercoin: u64, // supercoin
    idlecoin: u64,  // idlecoin
}

#[derive(Copy, Clone, Debug)]
struct Miner {
    miner_id: u64,  // miner address ID
    wallet_id: u64, // wallet address ID
    level: u64,     // current level
    cps: u64,       // coin-per-second
    inc: u64,       // Incrementor value
    pow: u64,       // Next level up value
}

#[derive(Debug)]
struct Connection {
    miner: Miner,      // Miner for connection
    stream: TcpStream, // TCP connection
    updates: String,   // Additional info for specific users
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
                    continue;
                }
            };
            let updates = format!("\nLogged in as: 0x{:016x}", miner.wallet_id).to_owned();
            let conn = Connection {
                miner,
                stream: s,
                updates,
            };

            let mut c = connections_close.lock().unwrap();
            c.push(conn);
        }
    });

    // Main loop
    let mut update = 0;
    loop {
        // Calculate miner performance and update stats
        process_miners(&connections, &wallets);

        // Send updates to all connected miners
        let mut msg = print_wallets(&connections, &wallets);

        // Roll dice for random actions
        action_miners(&connections, &mut msg);

        if update % 2 == 0 {
            // Send wallet updates to all connections every 2 seconds
            send_updates_to_all(msg, &connections);
            update = 0;
        }

        // Sleep from all that hard work
        sleep(Duration::from_secs(1));
        update += 1;
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
    for w in wals.iter() {
        if w.id == wallet_id {
            found = true;
        }
    }
    if !found {
        wals.push(Wallet {
            id: wallet_id,
            supercoin: 0,
            idlecoin: 0,
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
    drop(cons);
    if num >= 3 {
        let msg = format!("User denied U:0x{:016x}, too many miners\n", wallet_id);
        print!("{}", msg);
        return Err(Error::new(
            ErrorKind::ConnectionRefused,
            format!(
                "Connection refused: Too many miners connected for user 0x{:016x}.",
                wallet_id
            ),
        ));
    }

    // Generate a random miner_id
    let mut rng = rand::thread_rng();
    let miner_id: u64 = rng.gen();

    println!(
        "User++ U:0x{:016x} M:0x{:016x} from: {:?}",
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
        msg += &format!(
            "[{:03}] Wallet 0x{:016x} Coins: {}:{}\n",
            gens.len() - i,
            g.id,
            g.supercoin,
            g.idlecoin,
        )
        .to_owned();
        for c in cons.iter() {
            if c.miner.wallet_id == g.id {
                msg += &format!(
                    "    [+] Miner 0x{:016x} Cps: {} Level: {}\n",
                    c.miner.miner_id, c.miner.cps, c.miner.level,
                )
                .to_owned();
            }
        }
    }
    drop(cons);

    msg
}

fn send_updates_to_all(input: String, connections: &Arc<Mutex<Vec<Connection>>>) {
    let mut cons = connections.lock().unwrap();
    let mut rem = Vec::<usize>::new();

    for (i, c) in cons.iter_mut().enumerate() {
        // Append updates to input message
        let mut msg = input.clone();
        msg.push_str(&c.updates);

        // Send message to connection
        if c.stream.write_all(msg.as_bytes()).is_err() {
            // If error, mark miner for disconnection
            rem.push(i);
            println!(
                "User-- U:0x{:016x} M:0x{:016x} from: {:?}",
                c.miner.wallet_id, c.miner.miner_id, c.stream
            );
        }
    }

    // Remove disconnected miners
    for i in rem.iter() {
        cons.remove(*i);
    }
}

fn action_miners(connections: &Arc<Mutex<Vec<Connection>>>, input: &mut String) {
    let mut rng = rand::thread_rng();
    let mut msg = "".to_string();

    let mut cons = connections.lock().unwrap();

    for c in cons.iter_mut() {
        let r: u16 = rng.gen();
        let x: u16 = r % 1000;

        if x == 0 {
            // 0.1 % chance
            c.miner.level = match c.miner.level.checked_sub(1) {
                Some(n) => {
                    msg.push_str(&format!(
                        " [!] Miner 0x{:016x} lost a level\n",
                        c.miner.miner_id
                    ));
                    n
                }
                None => 0,
            };
        } else if x <= 5 {
            // 0.5 % chance
            c.miner.level = match c.miner.level.checked_add(1) {
                Some(n) => {
                    msg.push_str(&format!(
                        " [!] Miner 0x{:016x} leveled up\n",
                        c.miner.miner_id
                    ));
                    n
                }
                None => u64::MAX,
            };
        } else if x <= 10 {
            // .5 % chance
            c.miner.cps += match c.miner.cps.checked_div(2) {
                Some(n) => n,
                None => u64::MAX,
            };
            msg.push_str(&format!(
                " [!] Miner 0x{:016x} gained 50% CPS boost\n",
                c.miner.miner_id
            ));
        }
    }

    if !msg.is_empty() {
        input.push_str(&format!("\nEvents:\n{}", msg));
    };
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
}

fn add_idlecoins(mut miner: &mut Wallet, new: u64) {
    miner.idlecoin = match miner.idlecoin.checked_add(new) {
        Some(c) => c,
        None => {
            miner.supercoin = match miner.supercoin.checked_add(1) {
                Some(s) => s,
                None => u64::MAX,
            };
            let x: u128 = (u128::from(miner.idlecoin) + u128::from(new)) % u128::from(u64::MAX);
            x as u64
        }
    };
}

fn miner_session(mut miner: &mut Miner) {
    // Level up
    if miner.cps >= miner.pow {
        miner.level = match miner.level.checked_add(1) {
            Some(n) => n,
            None => u64::MAX,
        };

        miner.inc = match miner.inc.checked_add(miner.level) {
            Some(n) => n,
            None => u64::MAX,
        };

        miner.pow = match miner.pow.checked_mul(10) {
            Some(n) => n,
            None => u64::MAX,
        };
    }

    // Increment cps
    if miner.cps != u64::MAX {
        miner.cps = match miner.cps.checked_add(miner.inc + miner.level) {
            Some(n) => n,
            None => u64::MAX,
        };
    }

    // Perform action, maybe (randomly)
    /*if !action(stream, &mut miner) {
        break;
    }*/
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
        return Err(Error::new(ErrorKind::ConnectionRefused, "No data to load"));
    }

    // Attempt to deserialize the json file data
    println!("Loading stats...");
    if let Ok(mut wallet) = serde_json::from_str(&j) {
        // Update the wallets struct
        let mut gens = wallets.lock().unwrap();
        gens.append(&mut wallet);
        println!("Successfully loaded stats file {}", SAVE);
    } else {
        return Err(Error::new(
            ErrorKind::ConnectionRefused,
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
