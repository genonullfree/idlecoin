use std::io::Error;
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time;

const PORT: u16 = 7654;

#[derive(Copy, Clone, Debug, PartialEq)]
struct CoinsGen {
    name: [u8; 1024],
    coin: u64, // total idlecoin
    iter: u64, // session iteration idlecoin
    gen: u64,  // generated idlecoin
}

fn main() {
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

    // Create global array of user generators
    let generators = Arc::new(Mutex::new(Vec::<CoinsGen>::new()));

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

fn login(
    mut stream: &TcpStream,
    generators: &Arc<Mutex<Vec<CoinsGen>>>,
) -> Result<CoinsGen, Error> {
    // Lock generators
    let gens = generators.lock().unwrap();

    // Request username
    let msg = format!(
        "Welcome to Idlecoin! There are {} current users.\nPlease enter your account: ",
        gens.len()
    );
    stream.write_all(msg.as_bytes())?;

    // Read username
    let mut name: [u8; 1024] = [0; 1024];
    let _ = stream.read(&mut name[..]).unwrap();

    // Look for user record
    for i in gens.deref() {
        if name == i.name {
            return Ok(*i);
        }
    }

    // Create new record
    Ok(CoinsGen {
        name,
        coin: 0,
        iter: 0,
        gen: 0,
    })
    // Unlock generators
}

fn update_generator(
    generators: &Arc<Mutex<Vec<CoinsGen>>>,
    mut coin: &mut CoinsGen,
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

fn session(mut stream: TcpStream, generators: Arc<Mutex<Vec<CoinsGen>>>) -> Result<(), Error> {
    // Allow user session to login
    let mut miner = login(&stream, &generators)?;
    //let initcoin = gen.coin;
    miner.gen = 1;
    miner.iter = 0;

    let mut inc = 1;
    let mut pow = 10;

    // Main loop
    loop {
        // Level up
        if miner.gen > pow {
            inc <<= 1;
            pow *= 10;
            update_generator(&generators, &mut miner)?;
            let msg = format!(
                "\n===\nIdlecoin generator upgrade:\ninc: {}\npow: {}\niter: {}\nTOTAL IDLECOIN: {}\n===\n",
                inc, pow, miner.gen, miner.coin
            );
            match stream.write_all(msg.as_bytes()) {
                Ok(_) => (),
                Err(_) => break,
            };
        }

        // Increment coins
        let msg = format!("\rIteration idlecoins: {}", miner.gen);
        match stream.write_all(msg.as_bytes()) {
            Ok(_) => (),
            Err(_) => break,
        };
        miner.gen += inc;

        // Rest from all that work
        sleep(time::Duration::from_millis(100));
    }

    update_generator(&generators, &mut miner)?;

    Ok(())
}
