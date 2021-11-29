use tokio::net::{TcpListener, TcpStream};

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


pub async fn start_server() {
    let listener = TcpListener::bind("0.0.0.0:80").await;
    match listener {
        Ok(listen) => {
            println!("SERVER START SUCCESSFUL");
            loop {
                let (socket, _) = listen.accept().await.unwrap();
                tokio::spawn(async {
                    process(socket).await;
                });
            }
        }
        Err (err) => {
            println!("{}", err);
        }
    }
    
}