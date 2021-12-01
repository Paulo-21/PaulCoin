use tokio::net::{ TcpListener, TcpStream, tcp::OwnedReadHalf, tcp::OwnedWriteHalf };
use std::fs::File;
use std::io::prelude::*;
use tokio::sync::{ mpsc };
use std::io;
//use std::net::SocketAddr;

#[derive(Debug)]
enum Commande {
    Send {
        value:String
    },
    AddStream {
        writer_stream : mpsc::Sender<Commande>
    }
    
}
mod server;
async fn stream_writer(stream : OwnedWriteHalf, mut rx : mpsc::Receiver<Commande>) {
    while let Some(cmd) = rx.recv().await {
        use Commande::*;
        match cmd {
            Send { value } => {
                loop {
                    stream.writable().await;
                    match stream.try_write(value.as_bytes()) {
                        Ok(n) => {
                            break;
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            return;
                        }
                    }
                }
            },
            _ => {
                continue;
            }
        }
    }
}
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


async fn process_receive(msg : Vec<u8>, tx_mpsc_manager : mpsc::Sender<Commande>) {
    let c: &[u8] = &msg;
    let str = String::from_utf8_lossy(c);
    tx_mpsc_manager.send(Commande::Send{value : String::from(str)}).await;
}
async fn manager ( mut rx_server: mpsc::Receiver<Commande>) {
    let mut writers: Vec<mpsc::Sender<Commande>> = Vec::new();
    while let Some(cmd) = rx_server.recv().await {
        use Commande::*;
        match cmd {
            Send { value } => {
                for tx in &writers {
                    tx.send(Commande::Send{value : value.clone()}).await;
                }
            },
            AddStream { writer_stream } => {
                writers.push(writer_stream);
            }
            _ => {
                continue;
            }
        }
    }
}

async fn process_new_connection (stream : TcpStream, tx_mpsc_manager: mpsc::Sender<Commande>) {
    let (reader, writer) = stream.into_split();
    let tx_mpsc_manager_clone = tx_mpsc_manager.clone();
    let (tx, rx) = mpsc::channel(32);
    tokio::spawn( async move {
        stream_writer(writer, rx).await;
    });
    tokio::spawn( async move {
        stream_reader(reader, tx_mpsc_manager_clone).await;
    });

    tx_mpsc_manager.send(Commande::AddStream{writer_stream : tx}).await;
}

async fn start_server(tx_mpsc_manager: mpsc::Sender<Commande>) {
    let listener = TcpListener::bind("0.0.0.0:80").await;
    match listener {
        Ok(listen) => {
            println!("SERVER START SUCCESSFUL");
            loop {
                let (socket, addr) = listen.accept().await.unwrap();
                let tx_mpsc_manager_clone =  tx_mpsc_manager.clone();
                println!("Receive connection from : {}", addr);
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
    let (tx_mpsc_manager ,  rx_mpsc_manager) : (mpsc::Sender<Commande>, mpsc::Receiver<Commande>) = mpsc::channel(32);
    let mut file = File::open("votant.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    let clients = contents.split(' ').collect::<Vec<_>>();
    
    let _manager = tokio::spawn(async move{
        manager(rx_mpsc_manager).await;
    });
    
    for ip in clients {
        let tx_mpsc_manager_clone = tx_mpsc_manager.clone();
        let a = String::from(ip).clone();
        tokio::spawn(async move {
            start_client(a, tx_mpsc_manager_clone).await;
        });
    }
    let server = tokio::spawn(start_server(tx_mpsc_manager) );
    server.await.unwrap();
    _manager.await;
}

async fn start_client(mut ip : String, tx_mpsc_manager : mpsc::Sender<Commande>) {
    ip.push_str(":80");
    match TcpStream::connect(&ip[..]).await {
        Ok(stream) => {
            if stream.local_addr().unwrap() == stream.peer_addr().unwrap() {
                return;
            }
            println!("Connected : {}", ip);
            let tx_mpsc_manager_clone =  tx_mpsc_manager.clone();
            tokio::spawn(async {
                process_new_connection(stream, tx_mpsc_manager_clone).await;
            });
        }
        Err(err) => { println!("{}", err); }
    }
    println!("client destroyed");
}