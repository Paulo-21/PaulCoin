use tokio::net::{TcpListener, TcpStream, tcp::OwnedReadHalf, tcp::OwnedWriteHalf};
use std::fs::File;
use std::io::prelude::*;
use tokio::sync::{mpsc, watch };
//use std::net::SocketAddr;

#[derive(Debug, Clone)]
enum Commande {
    Send {
        value:String
    },
    Receive {
        value:String
    }
}
mod server;

async fn stream_reader(stream : OwnedReadHalf, tx_mpsc_manager: mpsc::Sender<Commande>) {
    loop {
        let mut msg = vec![0;1024];
        stream.readable().await;

        match stream.try_read(&mut msg) {
            Ok(0) => { 
                println!("connection close");
                break; 
            }
            Ok(n) => {
                /*if stream.local_addr().is_ok() {
                    continue;
                }*/
                msg.truncate(n);
                let tx_mpsc_manager_clone = tx_mpsc_manager.clone();
                tokio::spawn(async move { 
                    process_receive(msg, tx_mpsc_manager_clone).await; 
                });
            }
            
            Err(e) => {
                continue;
            }
        }
    }
}
async fn stream_writer(stream : OwnedWriteHalf, mut rx_watch_manager: watch::Receiver<Commande>) {
    while rx_watch_manager.changed().await.is_ok() {
            use Commande::*;
            //Je commence à bien maitriser
            let c = { (*rx_watch_manager.borrow()).clone() };
            match c {
                Send {value} => {
                    loop {
                        stream.writable().await;
                        //Optimisation à faire
                        //Value.as_bytes depuis le manager ou en tant que valeur de Command
                        match stream.try_write(value.as_bytes()) {
                            Ok(n) => { break; }
                            Err(_e) => { continue; }
                        }
                    }
                },
                _ => { continue; }
            }
             
        }
}


async fn process_receive(msg : Vec<u8>, tx_mpsc_manager : mpsc::Sender<Commande>) {
    let c: &[u8] = &msg;
    let str = String::from_utf8_lossy(c);
    tx_mpsc_manager.send(Commande::Send{value : String::from(str)}).await;
    
}
async fn manager ( mut rx_server: mpsc::Receiver<Commande> , mut tx_watch : watch::Sender<Commande>) {
    while let Some(cmd) = rx_server.recv().await {
        use Commande::*;
        match cmd {
            Send { value } => {
                println!("Manager Receive : {}", value);
                tx_watch.send(Send { value : value});
            }
            _ => {
                continue;
            }
        }
    }
}

async fn process_new_connection (mut stream : TcpStream, tx_mpsc_manager: mpsc::Sender<Commande>, rx_watch_manager: watch::Receiver<Commande>) {
    let (reader, writer) = stream.into_split();
    let tx_mpsc_manager_clone = tx_mpsc_manager.clone();
    tokio::spawn(async move {
        stream_writer(writer, rx_watch_manager).await;
    });
    tokio::spawn( async move {
        stream_reader(reader, tx_mpsc_manager_clone).await;
    });
    tx_mpsc_manager.send(Commande::Send{value: String::from("ADD")});
}

async fn start_server(tx_mpsc_manager: mpsc::Sender<Commande>, rx_watch_manager: watch::Receiver<Commande>) {
    let listener = TcpListener::bind("0.0.0.0:80").await;
    match listener {
        Ok(listen) => {
            println!("SERVER START SUCCESSFUL");
            loop {
                let (socket, addr) = listen.accept().await.unwrap();
                let tx_mpsc_manager_clone =  tx_mpsc_manager.clone();
                let rx_watch_manager_clone = rx_watch_manager.clone();
                println!("Receive connection from : {}", addr);
                tokio::spawn(async {
                    process_new_connection(socket, tx_mpsc_manager_clone, rx_watch_manager_clone).await;
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
    let (tx_watch ,  rx_watch) : (watch::Sender<Commande>, watch::Receiver<Commande>) = watch::channel(Commande::Send{value:String::from("Bonjour")});
    let (tx_mpsc_manager ,  rx_mpsc_manager) : (mpsc::Sender<Commande>, mpsc::Receiver<Commande>) = mpsc::channel(32);
    let mut file = File::open("votant.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    let clients = contents.split(' ').collect::<Vec<_>>();
    
    let _manager = tokio::spawn(async move{
        manager(rx_mpsc_manager, tx_watch).await;
    });
    
    for ip in clients {
        let tx_mpsc_manager_clone = tx_mpsc_manager.clone();
        let rx_watch_clone = rx_watch.clone();
        let a = String::from(ip).clone();
        tokio::spawn(async move {
            start_client(a, tx_mpsc_manager_clone, rx_watch_clone).await;
        });
    }
    let server = tokio::spawn(start_server(tx_mpsc_manager, rx_watch) );
    server.await.unwrap();
    _manager.await;
}

async fn start_client(mut ip : String, tx_mpsc_manager : mpsc::Sender<Commande>, rx_watch_manager: watch::Receiver<Commande>) {
    ip.push_str(":80");
    match TcpStream::connect(&ip[..]).await {
        Ok(stream) => {
            println!("Connected : {}", ip);
            let tx_mpsc_manager_clone =  tx_mpsc_manager.clone();
            let rx_watch_manager_clone = rx_watch_manager.clone();
            tokio::spawn(async {
                process_new_connection(stream, tx_mpsc_manager_clone, rx_watch_manager_clone).await;
            });
        }
        Err(err) => { println!("{}", err); }
    }
    println!("client destroyed");
}