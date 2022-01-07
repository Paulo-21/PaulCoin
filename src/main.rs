
use std::fs::File;
use std::io::prelude::*;
use tokio::sync::{ mpsc };
use std::env;
use std::collections::BTreeMap;
//use console_subscriber;
//use std::net::SocketAddr;
mod block;
mod node;
#[tokio::main]
async fn main() {
    //console_subscriber::init();
    let arg = env::args().last();
    let mut movie_reviews : BTreeMap<Vec<u8>, block::Block> = BTreeMap::new();
    let (tx_mpsc_manager ,  rx_mpsc_manager) : (mpsc::Sender<node::Commande>, mpsc::Receiver<node::Commande>) = mpsc::channel(32);
    let mut file = File::open("votant.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    println!("{}", contents);
    let clients = contents.split(' ').collect::<Vec<_>>();
    
    let _manager = tokio::spawn(async move{
        node::manager(rx_mpsc_manager).await;
    });
    
    for ip in clients {
        let tx_mpsc_manager_clone = tx_mpsc_manager.clone();
        let a = String::from(ip).clone();
        let b =  arg.clone();
        tokio::spawn(async move {
            node::start_client(a, tx_mpsc_manager_clone, b).await;
        });
    }
    let server = tokio::spawn( node::start_server(tx_mpsc_manager) );
    server.await.unwrap();
    _manager.await.unwrap();
}