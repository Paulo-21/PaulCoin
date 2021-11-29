use tokio::net::{TcpListener, TcpStream, tcp::OwnedReadHalf, tcp::OwnedWriteHalf};
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

async fn stream_reader(stream : OwnedReadHalf) {

}
async fn stream_writer(stream : OwnedWriteHalf) {

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
async fn process_new_connection (mut stream : TcpStream, tx_mpsc_manager: mpsc::Sender<Commande>) {
    let (tx, rx_mpsc_receive_loop) : (mpsc::Sender<Commande>, mpsc::Receiver<Commande>) = mpsc::channel(20);
    let (reader, writer) = stream.into_split();
    tokio::spawn(async move {
        stream_writer(writer).await;
    });
    tokio::spawn( async move {
        stream_reader(reader).await;
    });
    tx_mpsc_manager.send(Commande::Send{value: String::from("ADD")});
}


async fn start_server(tx_mpsc_manager: mpsc::Sender<Commande>) {
    let listener = TcpListener::bind("0.0.0.0:80").await;
    match listener {
        Ok(listen) => {
            println!("SERVER START SUCCESSFUL");
            loop {
                let (socket, _) = listen.accept().await.unwrap();
                let tx_mpsc_manager_clone =  tx_mpsc_manager.clone();
                tokio::spawn(async {
                    process_new_connection(socket, tx_mpsc_manager_clone).await;
                });
            }
        }
        Err (err) => {
            println!("{}", err);
        }
    }
    
}
#[tokio::main]
async fn main() {
    
    let (tx_watch , rx_watch) : (watch::Sender<Commande>, watch::Receiver<Commande>) = watch::channel(Commande::Send{value:String::from("Bonjour")});
    let (tx_mpsc_manager , rx_mpsc_manager) : (mpsc::Sender<Commande>, mpsc::Receiver<Commande>) = mpsc::channel(32);
    let mut file = File::open("votant.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    let clients = contents.split(' ').collect::<Vec<_>>();
    
    let _manager = tokio::spawn(manager(rx_mpsc_manager, tx_watch) );
    //let mut client_spanw = Vec::new();
    
    /*for ip in clients {
        let tx2 = tx.clone();
        let a = String::from(ip).clone();
        client_spanw.push( tokio::spawn(start_client(a, tx2)));
    }*/
    let server = tokio::spawn(start_server(tx_mpsc_manager) );
    server.await.unwrap();
    /*for x in client_spanw {
        x.await;
    }*/
    _manager.await;
    
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

pub async fn process(stream: TcpStream) {

    let mut msg = vec![0;1024];
    loop {
        stream.readable().await;

        match stream.try_read(&mut msg) {
            Ok(n) => {
                msg.truncate(n);
                break;
            }
            Err(e) => {
                continue;
            }
        }
    }
    println!("Receive = {:?}", msg);
}