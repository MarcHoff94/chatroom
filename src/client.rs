use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let stdin = io::stdin();

    let mut ip = String::new();

    println!("Enter Chatserver IP to connect: ");
    stdin.read_line(&mut ip)?;
    let trimmed_ip = ip.trim();
    let mut stream = TcpStream::connect(trimmed_ip)?;
    println!("Connected to server at {}", trimmed_ip); 

    stream.set_nonblocking(true)?;

    loop {
        println!("Enter a username:");
        let mut username = String::new();
        stdin.read_line(&mut username)?;
        if username.len() <= 16 {
            println!("{}", username);
            let trimmed_username = username.trim();

            println!("Your username now is: {}", trimmed_username);
            stream.write(trimmed_username.as_bytes());
            break;
        }
        println!("Please choose a username with max. 16 letters")
        
    }

    let mut read_stream = stream.try_clone()?;
    thread::spawn(move || {
        let mut buffer = [0; 512]; 
        loop {
            match read_stream.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    if let Ok(mut received_msg) = String::from_utf8(buffer[..bytes_read].to_vec()) {
                        let message = received_msg.split_off(9);
                        let id_sender = received_msg.split_off(3);
                        println!("{}: {}",id_sender, message.trim());
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




