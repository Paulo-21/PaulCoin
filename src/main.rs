use tokio::net::{TcpListener, TcpStream};
use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use std::net::SocketAddr;

mod server;

async fn manager ( mut all_tcp : HashMap<SocketAddr, TcpStream> , mut rx: Receiver<TcpStream> ) {
        while let Some(stream) = rx.recv().await {
            loop {
                stream.writable().await;
                match stream.try_write(b"hello world") {
                    Ok(n) => { break; }
                    Err(_e) => { continue; }
                }
            }
            all_tcp.insert(stream.peer_addr().unwrap(), stream);
            println!("Hello");
        }
        println!("Start show all tcp");
        for (x, stream) in all_tcp {
            println!("{:?} : {:?}", x, stream.local_addr());
            
        }
}
async fn start_client(mut ip : String, tx  : Sender<TcpStream>) {
    ip.push_str(":80");
    let mut stream = TcpStream::connect(&ip[..]).await;
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
    drop(tx);
    println!("client destroyed");
}
#[tokio::main]
async fn main() {
    let mut all_tcp: HashMap<SocketAddr, TcpStream> = HashMap::new();
    let (tx , mut rx) : (Sender<TcpStream>, Receiver<TcpStream>) = mpsc::channel(32);
    let mut file = File::open("votant.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    println!("{}", contents);
    let clients = contents.split(' ').collect::<Vec<_>>();
    
    let mut client_spanw = Vec::new();
    let _manager = tokio::spawn(manager(all_tcp, rx));
    
    for ip in clients {
        let tx2 = tx.clone();
        let a = String::from(ip).clone();
        client_spanw.push(tokio::spawn( start_client(a, tx2) ));
    }
    let server = tokio::spawn(server::start_server());
    server.await;
    for x in client_spanw {
        x.await;
    }
    _manager.await;
    
}