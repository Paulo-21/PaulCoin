use tokio::net::{TcpListener, TcpStream};
use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

mod server;
async fn start_client(mut ip : String) {
    ip.push_str(":80");
    let mut client = TcpStream::connect(&ip[..]).await;
    match client {
        Ok(c) => {
            println!("Connected : {}", ip);
            loop {
                c.writable().await;
                match c.try_write(b"hello world") {
                    Ok(n) => {
                        break;
                    }
                    Err(_e) => {
                        continue;
                    }
                }
            }
        }
        Err(err) => { println!("{}", err); }
    }
    
}
#[tokio::main]
async fn main() {
    let all_tcp: HashMap<&str, TcpStream> = HashMap::new();
    
    let mut file = File::open("votant.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    println!("{}", contents);
    let clients = contents.split(' ').collect::<Vec<_>>();
    
    let mut client_spanw = Vec::new();

    for ip in clients {
        let a = String::from(ip).clone();
        client_spanw.push(tokio::spawn( start_client(a) ));
    }
    let server = tokio::spawn(server::start_server());
    server.await;
    for x in client_spanw {
        x.await;
    }
}