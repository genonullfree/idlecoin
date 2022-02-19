use std::io::Error;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::thread::sleep;
use std::time;

const PORT: u16 = 7654;

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

    for stream in listener.incoming() {
        let s = match stream {
            Ok(s) => s,
            _ => continue,
        };
        thread::spawn(move || {
            match session(s) {
                Ok(_) => (),
                Err(s) => println!("Err: {}", s),
            };
        });
    }
}

fn session(mut stream: TcpStream) -> Result<(), Error> {
    let mut coin: u64 = 0;
    let mut inc = 1;
    let mut pow = 10;
    loop {
        if coin % pow == 0 {
            inc <<= 1;
            pow *= 10;
            let msg = format!("Idlecoin stat upgrade:\ninc: {}\npow: {}\n", inc, pow);
            stream.write_all(msg.as_bytes())?;
        }

        let msg = format!("\ridlecoin: {}", coin);
        stream.write_all(msg.as_bytes())?;
        sleep(time::Duration::from_millis(100));
        coin += inc;
    }
}
