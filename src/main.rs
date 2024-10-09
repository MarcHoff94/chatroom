use std::{
    collections::HashMap, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is running on 127.0.0.1:8080");

    let chatroom = Arc::new(Mutex::new(Chatroom::new(1)));
    let mut id = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                id += 1;
                // Insert the new participant in the chatroom
                let chatroom_clone = Arc::clone(&chatroom);
                let stream = Arc::new(Mutex::new(stream));
                chatroom_clone.lock().unwrap().participants.insert(id, Arc::clone(&stream));

                // Spawn a new thread for each client
                thread::spawn(move || {
                    handle_client(chatroom_clone, stream, id);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(chatroom: Arc<Mutex<Chatroom>>, stream: Arc<Mutex<TcpStream>>, id: u8) {
    let mut reader = BufReader::new(stream.lock().unwrap().try_clone().unwrap());

    loop {
        let mut buffer = String::new();

        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("Client {} disconnected", id);
                chatroom.lock().unwrap().remove_participant(id);
                break;
            }
            Ok(_) => {
                println!("Client {} sent: {}", id, buffer.trim());

                chatroom.lock().unwrap().broadcast_message(buffer.as_bytes(), id);
            }
            Err(e) => {
                println!("Error reading from client {}: {}", id, e);
                chatroom.lock().unwrap().remove_participant(id);
                break;
            }
        }
    }
}

struct Chatroom {
    id: u8,
    participants: HashMap<u8, Arc<Mutex<TcpStream>>>,
}

impl Chatroom {
    fn new(id: u8) -> Chatroom {
        Chatroom {
            id,
            participants: HashMap::new(),
        }
    }

    fn broadcast_message(&self, message: &[u8], sender_id: u8) {
        for (id, participant) in &self.participants {
            if *id != sender_id {
                let mut stream = participant.lock().unwrap();
                if let Err(e) = stream.write_all(message) {
                    println!("Error sending message to client {}: {}", id, e);
                }
            }
        }
    }

    fn remove_participant(&mut self, id: u8) {
        self.participants.remove(&id);
        println!("Removed client {} from chatroom", id);
    }
}
