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
    coin: u64,
    inc: u64,
    pow: u64,
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
        inc: 1,
        pow: 10,
    })
    // Unlock generators
}

fn session(mut stream: TcpStream, generators: Arc<Mutex<Vec<CoinsGen>>>) -> Result<(), Error> {
    // Allow user session to login
    let mut gen = login(&stream, &generators)?;

    // Main loop
    loop {
        // Level up
        if gen.coin % gen.pow == 0 {
            gen.inc <<= 1;
            gen.pow *= 10;
            let msg = format!(
                "\nIdlecoin stat upgrade:\ninc: {}\npow: {}\n",
                gen.inc, gen.pow
            );
            match stream.write_all(msg.as_bytes()) {
                Ok(_) => (),
                Err(_) => break,
            };
        }

        // Increment coins
        let msg = format!("\ridlecoin: {}", gen.coin);
        match stream.write_all(msg.as_bytes()) {
            Ok(_) => (),
            Err(_) => break,
        };
        gen.coin += gen.inc;

        // Rest from all that work
        sleep(time::Duration::from_millis(100));
    }

    // Lock generators
    let mut gens = generators.lock().unwrap();
    gens.retain(|x| x.name != gen.name);
    gens.push(gen);
    drop(gens);
    // Unlock generators

    Ok(())
}
