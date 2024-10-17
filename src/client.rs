use std::fmt::format;
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
    let mut username = String::new();
    loop {
        println!("Enter a username:");
        // let mut username = String::new();
        stdin.read_line(&mut username)?;
        if username.len() <= 16 {
            println!("Your username now is: {}", username);
            match format_msg(MessageType::LOGIN, username.as_str().trim(), "I joined the chat!") {
                Ok(msg) => {
                    stream.write_all(msg.as_bytes());
                    break;
                }
                Err(e) => println!("{}",e)
            }
        }        
    }

    let mut read_stream = stream.try_clone()?;
    thread::spawn(move || {
        let mut buffer = [0; 512]; 
        loop {
            match read_stream.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    if let Ok(mut received_msg) = String::from_utf8(buffer[..bytes_read].to_vec()) {
                        println!("{}", received_msg);
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
    println!("Enter a message (type '/q' to exit): ");
    loop {
        input.clear();
        stdin.read_line(&mut input)?;

        let mut trimmed_input = input.trim();
        if trimmed_input == "/q" {
            break;
        }

        let msg = format_msg(MessageType::CHATMESSAGE, username.as_str().trim(), trimmed_input).unwrap();

        stream.write_all( msg.as_bytes())?;
    }

    Ok(())
}

pub enum MessageType {
    LOGIN,
    LOGOUT,
    CHATMESSAGE,
}
pub fn format_msg<'a>(msgtype: MessageType, username: &'a str, content: &'a str) -> Result<String, &'a str> {
    let mut msg = match msgtype {
        MessageType::LOGIN => String::from("100;"),
        MessageType::LOGOUT => String::from("101;"),
        MessageType::CHATMESSAGE => String::from("200;"),
    };
    if username.len() > 16 {
        return Err("Please choose a username with max. 16 letters") 
    } else if username.contains(";") {
        return Err("Illegal character \";\" was used")
    }
    msg.push_str(username);
    msg.push_str(";");
    msg.push_str(content);
    msg.push_str("\n");
    Ok(msg)
    
} 