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

    let generators = Arc::new(Mutex::new(Vec::<CoinsGen>::new()));

    for stream in listener.incoming() {
        let s = match stream {
            Ok(s) => s,
            _ => continue,
        };
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
    let gens = generators.lock().unwrap();
    let msg = format!(
        "Welcome to Idlecoin! There are {} current users.\nPlease enter your account: ",
        gens.len()
    );
    stream.write_all(msg.as_bytes())?;
    let mut name: [u8; 1024] = [0; 1024];
    let _ = stream.read(&mut name[..]).unwrap();

    for i in gens.deref() {
        if name == i.name {
            return Ok(*i);
        }
    }

    Ok(CoinsGen {
        name,
        coin: 0,
        inc: 1,
        pow: 10,
    })
}

fn session(mut stream: TcpStream, generators: Arc<Mutex<Vec<CoinsGen>>>) -> Result<(), Error> {
    let mut gen = login(&stream, &generators)?;

    loop {
        if gen.coin % gen.pow == 0 {
            gen.inc <<= 1;
            gen.pow *= 10;
            let msg = format!(
                "Idlecoin stat upgrade:\ninc: {}\npow: {}\n",
                gen.inc, gen.pow
            );
            match stream.write_all(msg.as_bytes()) {
                Ok(_) => (),
                Err(_) => break,
            };
        }

        let msg = format!("\ridlecoin: {}", gen.coin);
        match stream.write_all(msg.as_bytes()) {
            Ok(_) => (),
            Err(_) => break,
        };
        gen.coin += gen.inc;

        sleep(time::Duration::from_millis(100));
    }

    let mut gens = generators.lock().unwrap();
    gens.retain(|x| x.name != gen.name);
    gens.push(gen);

    Ok(())
}
