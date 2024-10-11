use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("Connected to server at 127.0.0.1:8080");

    
    stream.set_nonblocking(true)?;

    
    let mut read_stream = stream.try_clone()?;
    thread::spawn(move || {
        let mut buffer = [0; 512]; 
        loop {
            match read_stream.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    if let Ok(message) = String::from_utf8(buffer[..bytes_read].to_vec()) {
                        println!("Received from server: {}", message.trim());
                    } else {
                        println!("Received non-UTF8 data from server");
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100)); 
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        input.clear();
        println!("Enter a message (type '/q' to exit): ");
        stdin.read_line(&mut input)?;

        let trimmed_input = input.trim();
        if trimmed_input == "/q" {
            break;
        }

        // Send user input to the server
        stream.write_all(format!("{}\n", trimmed_input).as_bytes())?;
    }

    Ok(())
}
