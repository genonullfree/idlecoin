use std::collections::HashMap;
use std::fs::File;
use std::hash::Hasher;
use std::io;
use std::io::{Error, ErrorKind};
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::net::{SocketAddr, TcpListener, TcpStream};
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

mod commands;
mod file;
mod miner;
mod utils;
mod wallet;

use crate::miner::*;
use crate::wallet::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const PORT: u16 = 7654;
const SAVE: &str = ".idlecoin";
const AUTOSAVE: usize = 300;
const ABS_MAX_MINERS: u64 = 12;
const ABS_MAX_EVENTS: usize = 5;
const IDLECOIN: &str = r"

 /$$       /$$ /$$                                  /$$
|__/      | $$| $$                                 |__/
 /$$  /$$$$$$$| $$  /$$$$$$   /$$$$$$$  /$$$$$$     /$$ /$$$$$$$
| $$ /$$__  $$| $$ /$$__  $$ /$$_____/ /$$__  $$   | $$| $$__  $$
| $$| $$  | $$| $$| $$$$$$$$| $$      | $$  \ $$   | $$| $$  \ $$
| $$| $$  | $$| $$| $$_____/| $$      | $$  | $$   | $$| $$  | $$
| $$|  $$$$$$$| $$|  $$$$$$$|  $$$$$$$|  $$$$$$//$$| $$| $$  | $$
|__/ \_______/|__/ \_______/ \_______/ \______/|__/|__/|__/  |__/
";
const BANNER: &str = "
Source: https://github.com/genonullfree/idlecoin

Please enter your username: ";
const TURDS: [u64; 1] = [0xe492ba332d614d88];

const CLR: &str = "\x1b[2J\x1b[;H";
const RST: &str = "\x1b[0m";
const RED: &str = "\x1b[1;31m";
const GREEN: &str = "\x1b[1;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[1;34m";
const PURPLE: &str = "\x1b[1;35m";
const CYAN: &str = "\x1b[1;36m";

#[derive(Debug)]
pub struct Connection {
    miner: Miner,           // Miner for connection
    stream: TcpStream,      // TCP connection
    updates: Vec<String>,   // Additional info for specific users
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
    let wallets_exit = Arc::clone(&wallets);
    let conns_exit = Arc::clone(&connections);
    thread::spawn(move || {
        for sig in signals.forever() {
            if sig == SIGINT {
                // On Ctrl-C, enter a shutdown message
                print!("\rEnter a shutdown message: ");
                io::stdout().flush().unwrap();
                let stdin = io::stdin();
                let input = &mut String::new();
                stdin.read_line(input).unwrap();

                // Print message locally
                let t: DateTime<Local> = Local::now();
                print!("[!] Shutdown message at {t}: {input}");

                // Lock the connections, and hold lock until exit so no updates can be made to the miners
                let mut cons = conns_exit.lock().unwrap();
                for c in cons.iter_mut() {
                    // Send out the shutdown message to all connected miners
                    if c.stream.write_all(format!("{}{}{}{}v{}\n\n{}[!] The Idlecoin server is shutting down.{}\nMessage: {}{}{}Timestamp: {}\n", CLR, YELLOW, IDLECOIN, RST, VERSION, RED, RST, RED, input,RST, t).as_bytes()).is_ok() {}
                }

                // Save the current stats file
                file::save_stats(&wallets_exit);

                // Exit idlecoin
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
                    } else {
                        println!("Error in login: {} from {:?}", e, s);
                    }
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

            // Set write timeout
            match s.set_write_timeout(Some(Duration::from_millis(500))) {
                Ok(_) => (),
                Err(e) => {
                    println!("Unable to set write timeout: {e}");
                    continue;
                }
            };

            let updates = vec![format!(
                "\nLogged in as Wallet: {}0x{:016x}{} Miner: {}0x{:08x}{}\n",
                PURPLE, miner.wallet_id, RST, BLUE, miner.miner_id, RST
            )
            .to_owned()];
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
    let mut counter = AUTOSAVE;
    loop {
        commands::read_inputs(&connections, &wallets, &mut action_updates);

        // Calculate miner performance and update stats
        miner::process_miners(&connections, &wallets);

        // Increment chronocoins for each live wallet
        increment_chrono(&connections, &wallets);

        // Send updates to all connected miners
        let mut msg = print_wallets(&connections, &wallets);

        // Roll dice for random actions
        miner::action_miners(&connections, &wallets, &mut action_updates);

        // Format the update messages
        format_msg(&mut msg, &mut action_updates);

        // Send wallet updates to all connections every 2 seconds
        send_updates_to_all(msg, &connections);

        // Sleep from all that hard work
        sleep(Duration::from_secs(1));

        // Autosave every so often
        counter -= 1;
        if counter == 0 {
            // Increment chronocoins for each live wallet
            increment_rando(&connections, &wallets);

            file::save_stats(&wallets);
            counter = AUTOSAVE;
        }
    }
}

fn login(
    mut stream: &TcpStream,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
    connections: &Arc<Mutex<Vec<Connection>>>,
) -> Result<Miner, Error> {
    // Request userid
    let msg = format!(
        "{}Welcome to{}{}{}v{}\n\n{}",
        CLR, YELLOW, IDLECOIN, RST, VERSION, BANNER
    );
    if stream.write_all(msg.as_bytes()).is_err() {
        return Err(Error::new(ErrorKind::ConnectionReset, "No write-back"));
    };

    // Read userid
    let mut id_raw: [u8; 1024] = [0; 1024];

    // Only read 0-1023 to have the end NULL so we can safely do the
    // \r\n => \n\0 conversion
    let len = match stream.read(&mut id_raw[..1023]) {
        Ok(l) => l,
        Err(e) => return Err(e),
    };
    if len == 0 {
        return Err(Error::new(ErrorKind::ConnectionReset, "Nothing read"));
    }

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

    // Check if incoming connection is a turd
    for t in TURDS.iter() {
        if *t == wallet_id {
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!(
                    "Wallet {}0x{:016x}{}: Don't be a turd",
                    PURPLE, wallet_id, RST
                ),
            ));
        }
    }

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
        wals.push(Wallet::new(wallet_id));
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
            "{}Connection refused{}: Too many miners connected for user {}0x{:016x}{} (max: {})",
            RED, RST, PURPLE, wallet_id, RST, max_miners,
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
    Ok(Miner::new(wallet_id, miner_id))
}

fn increment_chrono(connections: &Arc<Mutex<Vec<Connection>>>, wallets: &Arc<Mutex<Vec<Wallet>>>) {
    let mut gens = wallets.lock().unwrap();
    let cons = connections.lock().unwrap();

    let mut live_wallets = HashMap::new();
    for c in cons.iter() {
        live_wallets.insert(c.miner.wallet_id, 1);
    }

    for g in gens.iter_mut() {
        if live_wallets.contains_key(&g.id) {
            g.inc_chronocoins();
        }
    }
}

fn increment_rando(connections: &Arc<Mutex<Vec<Connection>>>, wallets: &Arc<Mutex<Vec<Wallet>>>) {
    let mut live_wallets = HashMap::new();

    let cons = connections.lock().unwrap();
    for c in cons.iter() {
        live_wallets.insert(c.miner.wallet_id as u64, 1);
    }
    drop(cons);

    let mut rng = rand::thread_rng();
    let random: usize = rng.gen();
    let winner_pos = random % live_wallets.len();

    let mut winner_id: u64 = 0;
    for (i, (k, _)) in live_wallets.iter().enumerate() {
        if i == winner_pos {
            winner_id = *k;
            break;
        }
    }

    println!("[^] 0x{:016x} won 16 randocoins", winner_id);

    let mut gens = wallets.lock().unwrap();
    for w in gens.iter_mut() {
        if w.id == winner_id {
            w.inc_randocoins();
        }
    }
}

fn print_wallets(
    connections: &Arc<Mutex<Vec<Connection>>>,
    wallets: &Arc<Mutex<Vec<Wallet>>>,
) -> String {
    let mut msg = format!("{}{}{}{}v{}\n\n", CLR, YELLOW, IDLECOIN, RST, VERSION);
    let mut gens = wallets.lock().unwrap().deref().clone();
    let mut cons = connections.lock().unwrap();

    gens.sort_by(|a, b| a.idlecoin.cmp(&b.idlecoin));
    gens.sort_by(|a, b| a.supercoin.cmp(&b.supercoin));

    for (i, g) in gens.iter().enumerate() {
        let mut min = String::new();
        let mut total_cps = 0u128;
        let mut miner_line: Vec<String> = vec!["  ".to_string()];
        let mut num = 0;
        for c in cons.iter_mut() {
            if c.miner.wallet_id == g.id {
                // Build purchase display
                if g.idlecoin > 1024 || g.supercoin > 1 {
                    c.purchases = vec!["Commands:\n".to_string()];
                    if c.miner.cps > 1024 {
                        c.purchases.push(
                            format!(
                                "{}b{}: Purchase 128 seconds of Miner Boost for {} idlecoin / {}B{}: Purchase Max for {} idlecoin\n",
                                RED, RST,
                                commands::boost_cost(c.miner.cps),
                                RED, RST,
                                commands::boost_max_cost(c.miner.cps, c.miner.boost),
                            )
                            .to_string(),
                        );
                    }
                    let miner_cost = commands::miner_cost(g.max_miners);
                    if g.max_miners < ABS_MAX_MINERS && (g.idlecoin > miner_cost || g.supercoin > 1)
                    {
                        c.purchases.push(format!(
                            "{}m{}: Purchase 1 Miner License for {} idlecoin\n",
                            RED, RST, miner_cost
                        ));
                    }
                }
                if g.chronocoin > commands::time_cost() {
                    c.purchases.push(format!("{}c{}: Purchase 1 hour of time travel for this miner for {} chronocoin / {}C{}: Purchase Max for {} chronocoin\n", RED, RST,commands::time_cost(), RED, RST,commands::time_max_cost(g.chronocoin)));
                }

                // Build miner display
                miner_line.push(c.miner.print());
                total_cps += c.miner.cps as u128;
                num += 1;
                if num > 3 {
                    for i in &miner_line {
                        min += i;
                    }
                    min += "\n";
                    num = 0;
                    miner_line = vec!["  ".to_string()];
                }
            }
        }
        if num > 0 {
            for i in &miner_line {
                min += i;
            }
            min += "\n";
        }

        let wal = &format!(
            "[{}{:03}/{:03}{}] {} Total Cps: {}{}{}\n",
            CYAN,
            gens.len() - i,
            gens.len(),
            RST,
            g.print(),
            GREEN,
            total_cps,
            RST,
        )
        .to_owned();

        if !min.is_empty() {
            msg += wal;
            msg += &format!("  [{}*{}] Miners:\n", BLUE, RST);
            msg += &min;
            msg += "\n";
        }
    }
    drop(cons);
    drop(gens);

    msg
}

fn format_msg(input: &mut String, actions: &mut Vec<String>) {
    if actions.is_empty() {
        return;
    }

    input.push_str("\nEvents:\n");

    if actions.len() > ABS_MAX_EVENTS {
        actions.resize(ABS_MAX_EVENTS, "".to_owned());
    };

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

        for p in &c.purchases {
            msg.push_str(p);
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
