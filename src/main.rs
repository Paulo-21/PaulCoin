use tokio::net::{ TcpListener, TcpStream, tcp::OwnedReadHalf, tcp::OwnedWriteHalf };
//use tokio::io::{self, AsyncWriteExt};
use std::fs::File;
use std::io::prelude::*;
use tokio::sync::{ mpsc };
use std::io;
use bytes::{BytesMut, BufMut, Buf};
use std::env;
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
enum Frame {
    Message(String),
}
fn get_u8(buffer : &mut dyn Buf) -> Option<u8>{
    if buffer.has_remaining() {
        let ret = buffer.chunk()[0];
        buffer.advance(1);
        Some(ret)
    }
    else {
        None
    }
    
}
impl Frame {
    fn check(buffer : &mut dyn Buf) -> Result<(), &str>{
        
        match get_u8(buffer) {
            Some(n) => {
                println!("get_u8 {}", n);
                Ok(())
            }
            None  => {
                Err("nothing")
            }
        }
    }
    fn parse(buffer : &mut dyn Buf) -> () {


    }
}/*
fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}*/
fn parse_frame(buffer : &mut dyn Buf) -> Option<Frame>{
    match Frame::check(buffer) {
        Ok(_) => {
            Frame::parse(buffer);
            //let line = get_line(src).to_vec();
            //let string = String::from_utf8(line);
            Some(Frame::Message(String::from("fe")))
        }
        Err(e) => {
            None
        }
    }

}
async fn stream_reader(stream : OwnedReadHalf, tx_mpsc_manager: mpsc::Sender<Commande>) {
    let mut buf = BytesMut::with_capacity(1024);
    
    loop {
        if let frame = parse_frame(&mut buf) {
            //return Ok(Some(frame));
        }
        
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return ;
            }
        }
    }
}
async fn stream_writer(stream : OwnedWriteHalf, mut rx : mpsc::Receiver<Commande>) {
    while let Some(cmd) = rx.recv().await {
        use Commande::*;
        match cmd {
            Send { value } => {
                loop {
                    let mut data = vec![b'S'];
                    let vec = value.as_bytes();
                    data.extend(vec);
                    stream.writable().await;

                    match stream.try_write(&data) {
                        Ok(n) => { break; }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { continue; }
                        Err(e) => {
                            println!("{}", e);
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
                    let v = value.clone();
                    tx.send(Commande::Send{value : v}).await;
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
    let arg = env::args().next();
        
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
        let b =  arg.clone();
        tokio::spawn(async move {
            start_client(a, tx_mpsc_manager_clone, b).await;
        });
    }
    let server = tokio::spawn( start_server(tx_mpsc_manager) );
    server.await.unwrap();
    _manager.await.unwrap();
}

async fn start_client(mut ip : String, tx_mpsc_manager : mpsc::Sender<Commande>, message : Option<String>) {
    ip.push_str(":80");
    match TcpStream::connect(&ip[..]).await {
        Ok(stream) => {
            
            println!("{}, {}", stream.local_addr().unwrap().ip() , stream.peer_addr().unwrap().ip() );
            if stream.local_addr().unwrap().ip() == stream.peer_addr().unwrap().ip() {
                return;
            }
            if message.is_some() {
                let a = message.unwrap();
                loop {
                    stream.writable().await;
                    match stream.try_write(a.as_bytes()) {
                        Ok(n) => { break; }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { continue; }
                        Err(e) => {
                            println!("Error : {}", e);
                            return;
                        }
                    }
                }
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