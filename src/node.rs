use tokio::net::{ TcpListener, TcpStream, tcp::OwnedReadHalf, tcp::OwnedWriteHalf };
use tokio::sync::{ mpsc };
use std::io;
use bytes::{BytesMut, Buf};

#[derive(Debug)]
pub enum Commande {
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
struct BufferParsing {
    buf : BytesMut,
    cursor : usize
}
fn get_u8(buffer : &mut BufferParsing) -> Option<u8> {
    if !buffer.buf.has_remaining() {
        return None;
    }
    Some(buffer.buf.chunk()[buffer.cursor])
}
fn get_str (buffer : &mut BufferParsing) -> Option<Vec<u8>> {
     let iter = buffer.buf.chunk();
    loop {
        if buffer.cursor == iter.len() {
            return None;
        }
        if iter[buffer.cursor] == 3 {
            let mut dst = vec![0;buffer.cursor];
            buffer.buf.copy_to_slice(&mut dst);
            buffer.buf.advance(1);
            buffer.cursor = 0;
            return Some(dst);
        }
        buffer.cursor += 1;
    }
}
fn parse_frame(buffer : &mut BufferParsing) -> Option<Frame>{
    //if let Some(_n) = get_u8(buffer) {
            if let Some(str) = get_str(buffer) {
                let a = String::from_utf8(str).unwrap();
                return Some(Frame::Message(a));
            }
        //}
        None
}
async fn stream_reader(stream : OwnedReadHalf, tx_mpsc_manager: mpsc::Sender<Commande>) {
    let mut buffer = BufferParsing { buf : BytesMut::with_capacity(1024) , cursor : 0};
    
    loop {
        if let Some(Frame::Message(message)) = parse_frame(&mut buffer) {
                let cloned = tx_mpsc_manager.clone();
                tokio::spawn(async move {
                    process_receive(message.as_bytes(), cloned).await;
                });
        }
        stream.readable().await;
        match stream.try_read_buf(&mut buffer.buf) {
            Ok(0) => break,
            Ok(_n) => { 
                /*
                let mut dst = vec![0;buffer.buf.len()];
                buffer.buf.copy_to_slice(&mut dst);
                let a = String::from_utf8(dst).unwrap();
                println!("{}", a);
                */
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { continue; }
            Err(_e) => { return ; }
        }
    }
}
async fn stream_writer(stream : OwnedWriteHalf, mut rx : mpsc::Receiver<Commande>) {
    while let Some(cmd) = rx.recv().await {
        use Commande::*;
        match cmd {
            Send { value } => {
                loop {
                    let mut data = value.clone().into_bytes();
                    data.extend([3]);
                    stream.writable().await;
                    match stream.try_write(&data) {
                        Ok(_n) => { break; }
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


async fn process_receive(msg : &[u8], tx_mpsc_manager : mpsc::Sender<Commande>) {
    let str = String::from_utf8_lossy(msg);
    let str2 = String::from_utf8_lossy(msg);
    //println!("Receive msg : {}", str);
    tx_mpsc_manager.send(Commande::Send{value : String::from(str)}).await;
    tx_mpsc_manager.send(Commande::Send{value : String::from(str2)}).await;
}
pub async fn manager ( mut rx_server: mpsc::Receiver<Commande>) {
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
            },
        }
    }
}

pub async fn process_new_connection (stream : TcpStream, tx_mpsc_manager: mpsc::Sender<Commande>) {
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

pub async fn start_server(tx_mpsc_manager: mpsc::Sender<Commande>) {
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
pub async fn start_client(mut ip : String, tx_mpsc_manager : mpsc::Sender<Commande>, message : Option<String>) {
    ip.push_str(":80");
    match TcpStream::connect(&ip[..]).await {
        Ok(stream) => {
            println!("{}, {}", stream.local_addr().unwrap().ip() , stream.peer_addr().unwrap().ip() );
            if stream.local_addr().unwrap().ip() == stream.peer_addr().unwrap().ip() {
                return;
            }
            
            if message.is_some() {
                let a = message.unwrap();
                println!("client : {}", a);
                let mut data = a.into_bytes();
                data.extend([3]);
                loop {
                    stream.writable().await;
                    match stream.try_write(&data) {
                        Ok(_n) => { break; }
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
}