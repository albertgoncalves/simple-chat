use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
enum Comm {
    Connect(Arc<TcpStream>),
    Close(Arc<TcpStream>),
    Broadcast(SocketAddr, String),
}

fn client(stream: &Arc<TcpStream>, sender: &Sender<Comm>) {
    println!("{stream:?}");
    let peer = stream.peer_addr().unwrap();
    sender.send(Comm::Connect(stream.clone())).unwrap();
    loop {
        let mut reader = BufReader::new(stream.as_ref());
        let mut string = String::new();
        match reader.read_line(&mut string) {
            Ok(0) | Err(_) => {
                sender.send(Comm::Close(stream.clone())).unwrap();
                return;
            }
            Ok(_) => sender.send(Comm::Broadcast(peer, string)).unwrap(),
        }
    }
}

fn server(receiver: &Receiver<Comm>) {
    let mut clients = HashMap::new();
    loop {
        let comm = receiver.recv().unwrap();
        println!("{comm:?}");
        match comm {
            Comm::Connect(stream) => {
                let peer = stream.peer_addr().unwrap();
                let _ = clients.insert(peer, stream);
            }
            Comm::Close(stream) => {
                let peer = stream.peer_addr().unwrap();
                clients.remove(&peer).unwrap();
            }
            Comm::Broadcast(from, string) => {
                let message = format!("{from}: {string}");
                for (to, stream) in &clients {
                    if from == *to {
                        continue;
                    }
                    stream.as_ref().write_all(message.as_bytes()).unwrap();
                    stream.as_ref().flush().unwrap();
                }
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    let (sender, receiver) = channel();
    thread::spawn(move || {
        server(&receiver);
    });
    for stream in listener.incoming() {
        let sender = sender.clone();
        thread::spawn(move || {
            client(&Arc::new(stream.unwrap()), &sender);
        });
    }
}
