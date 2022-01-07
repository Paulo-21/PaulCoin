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
fn get_u8(buffer : &mut dyn Buf) -> Option<u8> {
    if !buffer.has_remaining() {
        return None;
    }
    let ret = buffer.chunk()[0];
    Some(ret)
    
}
fn get_str (buffer : &mut dyn Buf) -> Option<String> {
    let mut i = 0;
    let mut flag = false;
    let mut a = String::new(); 
    for x in buffer.chunk() {
        a.push(*x as char);
        if *x == 3 {
            flag = true;
            break;
        }
        i+=1;
    }
    
    if flag {
        a.remove(0);
        a.pop();
        buffer.advance(i+1);
        return Some(a);
    }
    None
}
impl Frame {
    fn check(buffer : &mut dyn Buf) -> Result<String, &str>{
        
        match get_u8(buffer) {
            Some(1) => { //begin
                //println!("Begin");
                match get_str(buffer) {
                    Some(str) => {
                        //println!("get_str : {}", str);
                        Ok(str)
                    },
                    None => {
                        Err("nothing")
                    }
                }
            }
            Some(3) => { //End
                //println!("End");
                Err("oups")
            },
            Some(n) => {
                println!("get_u8 {}", n);
                Err("dinguerie")
            }
            None  => {
                Err("nothing")
            }
        }
    }
    
}
fn parse_frame(buffer : &mut dyn Buf) -> Option<Frame>{
    match Frame::check(buffer) {
        Ok(frame) => {
            Some(Frame::Message(frame))
        }
        Err(e) => { None }
    }
}
async fn stream_reader(stream : OwnedReadHalf, tx_mpsc_manager: mpsc::Sender<Commande>) {
    let mut buf = BytesMut::with_capacity(1024);
    
    loop {
        match parse_frame(&mut buf) {
            Some(Frame::Message(message)) => {
                let cloned = tx_mpsc_manager.clone();
                tokio::spawn(async move {
                    process_receive(message.as_bytes(), cloned).await;
                });
            }
            None => {

            }
            
        }
        //println!("Buffer : {:?} ", buf);
        stream.readable().await;
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(_n) => {
                //println!("read {} bytes", n);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
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
                    let mut data = vec![1];
                    let vec = value.as_bytes();
                    data.extend(vec);
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
    //println!("Process msg : {}", str);
    tx_mpsc_manager.send(Commande::Send{value : String::from(str)}).await;
}
pub async fn manager ( mut rx_server: mpsc::Receiver<Commande>) {
    let mut writers: Vec<mpsc::Sender<Commande>> = Vec::new();
    while let Some(cmd) = rx_server.recv().await {
        use Commande::*;
        match cmd {
            Send { value } => {
                for tx in &writers {
                    let v = value.clone();
                    //tx.send(Commande::Send{value : v}).await;
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
            /*if stream.local_addr().unwrap().ip() == stream.peer_addr().unwrap().ip() {
                return;
            }*/
            if message.is_some() {
                let a = message.unwrap();
                let mut data = vec![1];
                let vec = a.as_bytes();
                data.extend(vec);
                data.extend([3]);
                println!("client : {}", a);
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
    //println!("client destroyed");
}