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

use serde::{Deserialize, Serialize};
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::hash::Hasher;
use xxhash_rust::xxh3;
//use serde_json::Result as SerdeResult;

const PORT: u16 = 7654;
const SAVE: &str = ".idlecoin";

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Wallet {
    name: u64,  // hash of name / wallet address
    coin: u64,  // total idlecoin
    iter: u64,  // session iteration idlecoin
    gen: u64,   // generated idlecoin
    level: u64, // current level
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
            _ => continue,
        };

        // Handle connection in new thread
        let generators_close = Arc::clone(&generators);
        thread::spawn(move || {
            match session(s, generators_close) {
                Ok(_) => (),
                Err(s) => println!("Err: {}", s),
            };
        });
    }
}

fn login(mut stream: &TcpStream, generators: &Arc<Mutex<Vec<Wallet>>>) -> Result<Wallet, Error> {
    // Lock generators
    let gens = generators.lock().unwrap();

    // Request username
    let msg = format!(
        "Welcome to Idlecoin! There are {} current users.\nPlease enter your account: ",
        gens.len()
    );
    stream.write_all(msg.as_bytes())?;

    // Read username
    let mut name_raw: [u8; 1024] = [0; 1024];
    let _ = stream.read(&mut name_raw[..]).unwrap();

    // Calculate the hash of the username
    let mut hash = xxh3::Xxh3::new();
    hash.write(&name_raw);
    let name = hash.finish();

    // Look for user record
    for i in gens.deref() {
        if name == i.name {
            return Ok(*i);
        }
    }

    // Create new record
    Ok(Wallet {
        name,
        coin: 0,
        iter: 0,
        gen: 0,
        level: 0,
    })
    // Unlock generators
}

fn update_generator(
    generators: &Arc<Mutex<Vec<Wallet>>>,
    mut coin: &mut Wallet,
) -> Result<(), Error> {
    let mut gens = generators.lock().unwrap();
    for i in gens.deref() {
        if i.name == coin.name {
            coin.coin = i.coin + (coin.gen - coin.iter);
            coin.iter = coin.gen;
        }
    }
    gens.retain(|x| x.name != coin.name);
    gens.push(*coin);
    drop(gens);

    Ok(())
}

fn print_generators(
    mut stream: &TcpStream,
    coin: &Wallet,
    generators: &Arc<Mutex<Vec<Wallet>>>,
) -> bool {
    let mut msg = "+++\n".to_string();
    let mut gens = generators.lock().unwrap().deref().clone();
    gens.sort_by(|a, b| b.coin.cmp(&a.coin));

    for g in gens {
        if g.name == coin.name {
            msg += &format!(
                "Wallet 0x{:016x} coins: {}, level: {} <= ***\n",
                coin.name, coin.coin, coin.level,
            )
            .to_owned()
        } else {
            msg += &format!("Wallet 0x{:016x} coins: {}\n", g.name, g.coin,).to_owned()
        };
    }
    if stream.write_all(msg.as_bytes()).is_err() {
        return false;
    }

    true
}

fn session(stream: TcpStream, generators: Arc<Mutex<Vec<Wallet>>>) -> Result<(), Error> {
    // Allow user session to login
    let mut miner = login(&stream, &generators)?;
    //let initcoin = gen.coin;
    miner.gen = 1;
    miner.iter = 0;
    miner.level = 1;

    let mut inc = 1;
    let mut pow = 10;

    let mut update = Instant::now();

    // Main loop
    loop {
        // Increment coins
        miner.gen += inc;
        update_generator(&generators, &mut miner)?;

        // Level up
        if miner.gen > pow {
            miner.level += 1;
            inc = 1 << miner.level;
            pow *= 10;
        }

        // Print updates
        if update.elapsed().as_secs() >= 1 {
            if !print_generators(&stream, &miner, &generators) {
                break;
            }
            update = Instant::now();
        }

        // Rest from all that work
        sleep(Duration::from_millis(100));
    }

    update_generator(&generators, &mut miner)?;

    Ok(())
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
    let mut c: Vec<Wallet> = serde_json::from_str(&j).unwrap();
    if c.is_empty() {
        println!("Failed to load {}", SAVE);
        return;
    }

    // Update the generators struct
    let mut gens = generators.lock().unwrap();
    gens.append(&mut c);
    drop(gens);

    println!("Successfully loaded stats file {}", SAVE);
}

fn save_stats(generators: Arc<Mutex<Vec<Wallet>>>) {
    // Serialize the stats data to json
    println!("Saving stats...");
    let gens = generators.lock().unwrap();
    let j = serde_json::to_string(&gens.deref()).unwrap();

    // Open the stats file for writing
    let mut file: File;
    file = match File::create(&SAVE) {
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
