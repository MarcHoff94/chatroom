use std::fmt::format;
use std::io::{self, Read, Stdin, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let stdin = io::stdin();

    let mut ip = String::new();

    println!("Enter Chatserver IP to connect: ");
    stdin.read_line(&mut ip)?;
    let trimmed_ip = ip.trim();
    let mut stream = Arc::new(Mutex::new(TcpStream::connect(trimmed_ip)?));
    println!("Connected to server at {}", trimmed_ip); 

    stream.lock().unwrap().set_nonblocking(true)?;
    let mut username = String::new();
    username = get_username(&stdin);
    await_server_response(Arc::clone(&stream), |bytes_read| {
        if let Ok(received_msg) = String::from_utf8(bytes_read.to_vec()) {
            if received_msg == "100" {
                return Ok(true)
            }
            Ok(false)
        } else {
            return Err("Received non-UTF8 data from server")
        }
    });
    let msg_username = format_msg(MessageType::LOGIN, username.trim(), "I have joined the chatroom!");
    stream.lock().unwrap().write_all(msg_username.as_bytes());
    


    let mut read_stream = Arc::clone(&stream);
    thread::spawn(move || {
        let mut buffer = [0; 512]; 
        loop {
            match read_stream.lock().unwrap().read(&mut buffer) {
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

        let trimmed_input = input.trim();
        if trimmed_input == "/q" {
            break;
        }

        let msg = format_msg(MessageType::CHATMESSAGE, username.as_str().trim(), trimmed_input);
        stream.lock().unwrap().write_all( msg.as_bytes())?;
    }

    Ok(())
}

pub enum MessageType {
    LOGIN,
    LOGOUT,
    CHATMESSAGE,
}
pub fn format_msg<'a>(msgtype: MessageType, username: &'a str, content: &'a str) -> String {
    let mut msg = match msgtype {
        MessageType::LOGIN => String::from("100;"),
        MessageType::LOGOUT => String::from("101;"),
        MessageType::CHATMESSAGE => String::from("200;"),
    };
    msg.push_str(username);
    msg.push_str(";");
    msg.push_str(content);
    msg.push_str("\n");
    return msg   
} 

fn get_username<'a>(io_input: &Stdin) -> String {
    let mut username = String::new();
    loop {
        username.clear();
        println!("Enter a username:");
        io_input.read_line(&mut username);
        match is_username_legal(username.as_str()) {
            Ok(_) => {
                return username;
            },
            Err(e) => println!("{}", e)
        }
    }
}

fn is_username_legal<'a>(username: &'a str) -> Result<(), &'a str> {
    if username.len() > 16 {
        return Err("Please choose a username with max. 16 letters") 
    } else if username.contains(";") {
        return Err("Illegal character \";\" was used")
    }
    Ok(())
}

fn await_server_response(read_stream: Arc<Mutex<TcpStream>>, condition_func: fn(bytes: &[u8])-> Result<bool,&'static str>) -> Result<(), &'static str> {
    let mut buffer = [0;512];
    loop {
        match read_stream.lock().unwrap().read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                match condition_func(&buffer[..bytes_read]) {
                    Ok(x) => {
                        if x == true {
                            break;
                        }
                    }
                    Err(e) => return Err(e)
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
    Ok(())
}