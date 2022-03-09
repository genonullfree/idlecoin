use std::fs::File;
use std::io::Error;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::hash::Hasher;
use xxhash_rust::xxh3;

const PORT: u16 = 7654;
const SAVE: &str = ".idlecoin";

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Wallet {
    id: u64,        // hash of id / wallet address
    supercoin: u64, // total idlecoin
    idlecoin: u64,  // decimal idlecoin
    level: u64,     // current level
    cps: u64,       // coin-per-second
}

fn main() {
    // Create global array of user generators
    let generators = Arc::new(Mutex::new(Vec::<Wallet>::new()));

    // Load previous stats file
    load_stats(&generators);

    // Bind network listener to port
    let listener = match TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT))) {
        Ok(l) => l,
        Err(_) => {
            println!(
                "Cannot bind to port: {}. Is idlecoin already running?",
                PORT
            );
            return;
        }
    };

    let mut signals = Signals::new(&[SIGINT]).unwrap();
    let generators_save = Arc::clone(&generators);
    thread::spawn(move || {
        for sig in signals.forever() {
            if sig == SIGINT {
                // Save the current stats file
                save_stats(generators_save);
                std::process::exit(0);
            }
        }
    });

    // Listen for connections
    for stream in listener.incoming() {
        let s = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        println!("Connection opened: {:?}", s);

        // Handle connection in new thread
        let generators_close = Arc::clone(&generators);
        thread::spawn(move || {
            let output = match session(&s, generators_close) {
                Ok(m) => format!("User logged out: 0x{:0x}", m.id),
                Err(s) => format!("Error: {}", s),
            };
            println!("Connection closed: {:?}, {}", s, output);
        });
    }
}

fn login(mut stream: &TcpStream, generators: &Arc<Mutex<Vec<Wallet>>>) -> Result<Wallet, Error> {
    // Request userid
    let msg = "Welcome to Idlecoin!\nPlease enter your account: ".to_string();
    stream.write_all(msg.as_bytes())?;

    // Read userid
    let mut id_raw: [u8; 1024] = [0; 1024];
    let _ = stream.read(&mut id_raw[..])?;

    // Calculate the hash of the userid
    let mut hash = xxh3::Xxh3::new();
    hash.write(&id_raw);
    let id = hash.finish();

    // Lock generators
    let gens = generators.lock().unwrap();

    println!("User joined: 0x{:08x}", id);
    // Look for user record
    for i in gens.deref() {
        if id == i.id {
            return Ok(*i);
        }
    }

    // Create new record
    Ok(Wallet {
        id,
        supercoin: 0,
        idlecoin: 0,
        level: 0,
        cps: 0,
    })
    // Unlock generators
}

fn update_generator(
    generators: &Arc<Mutex<Vec<Wallet>>>,
    mut coin: &mut Wallet,
) -> Result<(), Error> {
    let mut gens = generators.lock().unwrap();
    for i in gens.deref() {
        if i.id == coin.id {
            coin.idlecoin = match i.idlecoin.checked_add(coin.cps) {
                Some(c) => c,
                None => {
                    coin.supercoin += 1;
                    let x: u128 =
                        (u128::from(i.idlecoin) + u128::from(coin.cps)) % u128::from(u64::MAX);
                    x as u64
                }
            };
        }
    }
    gens.retain(|x| x.id != coin.id);
    gens.push(*coin);

    Ok(())
}

fn print_generators(
    mut stream: &TcpStream,
    coin: &Wallet,
    generators: &Arc<Mutex<Vec<Wallet>>>,
) -> bool {
    let mut msg = "+++\n".to_string();
    let mut gens = generators.lock().unwrap().deref().clone();
    gens.sort_by(|a, b| a.idlecoin.cmp(&b.idlecoin));
    gens.sort_by(|a, b| a.supercoin.cmp(&b.supercoin));

    for (i, g) in gens.iter().enumerate() {
        if g.id == coin.id {
            msg += &format!(
                "[{:03}] Wallet 0x{:016x} Coins: {}:{}, CPS: {}, level: {} <= ***\n",
                gens.len() - i,
                coin.id,
                coin.supercoin,
                coin.idlecoin,
                coin.cps,
                coin.level,
            )
            .to_owned()
        } else {
            msg += &format!(
                "[{:03}] Wallet 0x{:016x} Coins: {}:{}, CPS: {}\n",
                gens.len() - i,
                g.id,
                g.supercoin,
                g.idlecoin,
                g.cps
            )
            .to_owned()
        };
    }
    if stream.write_all(msg.as_bytes()).is_err() {
        return false;
    }

    true
}

fn action(mut stream: &TcpStream, mut miner: &mut Wallet) -> bool {
    let mut rng = rand::thread_rng();
    let x: u16 = rng.gen();
    let mut msg = "".to_string();

    if x % 40000 == 0 {
        msg = "Oh no! You've lost a level!\n".to_string();
        miner.level -= 1;
    } else if x % 1000 == 0 {
        msg = "Congrats! You've leveled up!\n".to_string();
        miner.level = match miner.level.checked_add(1) {
            Some(n) => n,
            None => u64::MAX,
        };
    } else if x % 100 == 0 {
        let prize = match miner.cps.checked_mul(10) {
            Some(n) => n,
            None => u64::MAX,
        };
        msg = format!("Congrats! You've won {} free idlecoins!\n", prize);
        miner.cps = match miner.cps.checked_add(prize) {
            Some(n) => n,
            None => u64::MAX,
        };
    }

    if !msg.is_empty() && stream.write_all(msg.as_bytes()).is_err() {
        return false;
    };
    true
}

fn session(stream: &TcpStream, generators: Arc<Mutex<Vec<Wallet>>>) -> Result<Wallet, Error> {
    // Allow user session to login
    let mut miner = login(stream, &generators)?;

    miner.level = 1;
    miner.cps = 0;

    let mut inc = 1;
    let mut pow = 100;

    let mut update = Instant::now();

    // Main loop
    loop {
        // Increment coins
        miner.cps = match miner.cps.checked_add(inc) {
            Some(n) => n,
            None => u64::MAX,
        };

        if !action(stream, &mut miner) {
            break;
        }

        update_generator(&generators, &mut miner)?;

        // Level up
        if miner.cps > pow {
            miner.level = match miner.level.checked_add(1) {
                Some(n) => n,
                None => u64::MAX,
            };
            inc = match inc.checked_add(1 << miner.level) {
                Some(n) => n,
                None => u64::MAX,
            };
            pow = match pow.checked_mul(10) {
                Some(n) => n,
                None => u64::MAX,
            };
        }

        // Print updates
        if update.elapsed().as_secs() >= 1 {
            if !print_generators(stream, &miner, &generators) {
                break;
            }
            //miner.cps = 0;
            update = Instant::now();
        }

        inc = match inc.checked_add(1) {
            Some(n) => n,
            None => u64::MAX,
        };
        // Rest from all that work
        sleep(Duration::from_millis(500));
    }

    update_generator(&generators, &mut miner)?;

    Ok(miner)
}

fn load_stats(generators: &Arc<Mutex<Vec<Wallet>>>) {
    let mut j = String::new();

    // Attempt to open and read the saved stats file
    let mut file = match File::open(&SAVE) {
        Ok(f) => f,
        Err(_) => {
            println!("No previous stats file found");
            return;
        }
    };
    file.read_to_string(&mut j).unwrap();

    // Exit if file is empty
    if j.is_empty() {
        println!("No data to load");
        return;
    }

    // Attempt to deserialize the json file data
    println!("Loading stats...");
    if let Ok(mut wallet) = serde_json::from_str(&j) {
        // Update the generators struct
        let mut gens = generators.lock().unwrap();
        gens.append(&mut wallet);
        println!("Successfully loaded stats file {}", SAVE);
    } else {
        println!("Failed to load {}", SAVE);
    }
}

fn save_stats(generators: Arc<Mutex<Vec<Wallet>>>) {
    // Serialize the stats data to json
    println!("Saving stats...");
    let gens = generators.lock().unwrap();
    let j = serde_json::to_string(&gens.deref()).unwrap();

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
