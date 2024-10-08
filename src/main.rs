use std::{
    io::{BufRead, BufReader, Write}, 
    net::{TcpListener, TcpStream}, 
    thread,
    time::Duration,
    sync::{Arc, Mutex},
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is running on 127.0.0.1:8080");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("Connection failed: {}",e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let mut buffer = String::new();

        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                println!("Received: {}", buffer.trim());

                stream.write_all(buffer.as_bytes()).expect("Failed to send message to Client");
            }
            Err(e) => {
                println!("Error reading from stream: {}", e);
                break;
            }
        }
    }
    thread::sleep(Duration::from_secs(1));
}
