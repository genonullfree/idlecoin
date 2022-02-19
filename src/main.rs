use std::time;
use std::net::Ipv4Addr;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::thread::sleep;
use std::io::Write;

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
            session(s);
        });
    }
}

fn session(mut stream: TcpStream) {
    let mut coin = 0;
    loop {
        let msg = format!("idlecoin: {}\r", coin);
        match stream.write_all(&msg.as_bytes()) {
            Ok(_) => (),
            Err(s) => {
                println!("Err: {}", s);
                return;
            },
        };
        sleep(time::Duration::from_millis(100));
        coin += 1;
    }
}
