use tokio::net::{TcpStream};
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use tokio::sync::{mpsc, watch };

use std::net::SocketAddr;

enum Commande {
    Send {
        value:String
    },
    Receive {
        value:String
    }
}
mod server;
async fn receive() {
    loop {

    }
}
async fn stream(stream : TcpStream, mut rx : mpsc::Receiver<Commande>) {
    while let Some(cmd) = rx.recv().await {
        use Commande::*;
        match cmd {
            Send { value } => {
                loop {
                    stream.writable().await;
                    match stream.try_write(value.as_bytes()) {
                        Ok(_n) => { break; }
                        Err(_e) => { continue; }
                    }
                }
            }
            Receive { value } => {
                let mut msg = vec![0;1024];
                loop {
                    stream.readable().await;
                    match stream.try_read(&mut msg) {
                        Ok(n) => {
                            msg.truncate(n);
                            tokio::spawn(async move {
                                process_receive(msg).await;
                            });
                            break;
                        }
                        Err(_e) => {
                            continue;
                        }
                    }
                }
            }
        }
    }
}
async fn process_receive(msg : Vec<u8>) {
    let c: &[u8] = &msg;
    println!("PROCESS : {}", String::from_utf8_lossy(c));
}
async fn manager ( mut rx_server: mpsc::Receiver<Commande> , mut tx_streams : watch::Sender<Commande>) {
    while let Some(stream) = rx_server.recv().await {
        /*loop {
            stream.writable().await;
            match stream.try_write(b"hello world") {
                Ok(_n) => { break; }
                Err(_e) => { continue; }
            }
        }*/
    }
}
async fn start_client(mut ip : String, tx  : mpsc::Sender<TcpStream>) {
    ip.push_str(":80");
    let stream = TcpStream::connect(&ip[..]).await;
    match stream {
        Ok(c) => {
            println!("Connected : {}", ip);
            /*loop {
                c.writable().await;
                match c.try_write(b"hello world") {
                    Ok(n) => { break; }
                    Err(_e) => { continue; }
                }
            }*/
            tx.send(c).await.unwrap();
        }
        Err(err) => { println!("{}", err); }
    }
    println!("client destroyed");
}
#[tokio::main]
async fn main() {
    
    let (tx_watch , rx_watch) : (watch::Sender<Commande>, watch::Receiver<Commande>) = watch::channel(Commande::Send{value:String::from("Bonjour")});
    let (tx_mpsc , rx_mpsc) : (mpsc::Sender<Commande>, mpsc::Receiver<Commande>) = mpsc::channel(32);
    let mut file = File::open("votant.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    let clients = contents.split(' ').collect::<Vec<_>>();
    
    //let mut client_spanw = Vec::new();
    let _manager = tokio::spawn(manager(rx_mpsc, tx_watch) );
    
    /*for ip in clients {
        let tx2 = tx.clone();
        let a = String::from(ip).clone();
        client_spanw.push( tokio::spawn(start_client(a, tx2)));
    }*/
    let server = tokio::spawn(server::start_server() );
    server.await.unwrap();
    /*for x in client_spanw {
        x.await;
    }*/
    _manager.await;
    
}