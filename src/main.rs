use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

enum Comm {
    ClientAdd(Arc<TcpStream>),
    ClientRemove(Arc<TcpStream>),
    Message(SocketAddr, String),
}

macro_rules! debug_println {
    ($($args:tt)*) => {
        #[cfg(debug_assertions)]
        {
            print!("{}:{}: ", file!(), line!());
            println!($($args)*);
        }
    };
}

fn client(stream: Arc<TcpStream>, sender: Sender<Comm>) {
    let peer = stream.peer_addr().unwrap();
    debug_println!("({peer}) Client joined");
    sender.send(Comm::ClientAdd(stream.clone())).unwrap();
    loop {
        let mut reader = BufReader::new(stream.as_ref());
        let mut string = String::new();
        match reader.read_line(&mut string) {
            Ok(0) | Err(_) => {
                sender.send(Comm::ClientRemove(stream.clone())).unwrap();
                debug_println!("({peer}) Client left");
                return;
            }
            Ok(_) => sender.send(Comm::Message(peer, string)).unwrap(),
        }
    }
}

fn server(receiver: Receiver<Comm>) {
    let mut clients = HashMap::new();
    loop {
        match receiver.recv().unwrap() {
            Comm::ClientAdd(stream) => {
                let peer = stream.peer_addr().unwrap();
                debug_println!("({peer}) Client added");
                let _ = clients.insert(peer, stream);
            }
            Comm::ClientRemove(stream) => {
                let peer = stream.peer_addr().unwrap();
                debug_println!("({peer}) Client removed");
                clients.remove(&peer).unwrap();
            }
            Comm::Message(from, string) => {
                debug_println!("({from}) Client sent a message");
                let message = format!("{from}: {string}");
                for (to, stream) in &clients {
                    if from == *to {
                        continue;
                    }
                    stream.as_ref().write_all(message.as_bytes()).unwrap();
                    stream.as_ref().flush().unwrap();
                }
                println!("{message}", message = message.trim_end());
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    let (sender, receiver) = channel();
    thread::spawn(move || {
        server(receiver);
    });
    for stream in listener.incoming() {
        let stream = Arc::new(stream.unwrap());
        let sender = sender.clone();
        thread::spawn(move || {
            client(stream, sender);
        });
    }
}
