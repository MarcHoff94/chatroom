use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server is running on 127.0.0.1:8080");

    let chatroom = Arc::new(Mutex::new(Chatroom::new(1)));
    let mut id = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                //can overflow...!
                id += 1;
                
                let chatroom_clone = Arc::clone(&chatroom);
                let stream = Arc::new(Mutex::new(stream));
                chatroom_clone.lock().unwrap().add_participant(id, Arc::clone(&stream));

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
    fn add_participant(&mut self, id: u8, stream: Arc<Mutex<TcpStream>>) {
        self.participants.insert(id, stream);
        let msg = format!("Client {} has entered the chatroom", id);
        self.broadcast_message(msg.as_bytes(), id);
    }
    fn remove_participant(&mut self, id: u8) {
        self.participants.remove(&id);
        let msg = format!("Removed client {} from chatroom", id);
        self.broadcast_message(msg.as_bytes(), id);
        println!("{}", msg);
    }
}

struct Lobby {
    logged_in: HashMap<u8, Arc<Mutex<TcpStream>>>,
    logged_out: Vec<u8>,
    chatrooms: HashMap<u8, Arc<Mutex<Chatroom>>>,
    new_id: u8
}
impl Lobby {
    fn new() -> Lobby {
        Lobby { logged_in: HashMap::new(), logged_out: Vec::new(), chatrooms: HashMap::new(), new_id: 0 }
    }
    fn log_in(&mut self, id:u8, stream: Arc<Mutex<TcpStream>>) -> Result<(), &str> {
        self.log_out(id).unwrap_or_else(|_| {});
        self.logged_in.insert(id, stream);
        Ok(())
    }
    fn log_out(&mut self, id: u8) -> Result<(), &str> {
        if self.logged_out.contains(&id) {
            let idx = self.logged_out.binary_search_by(|x| x.cmp(&id)).unwrap();
            self.logged_out.remove(idx);
        } else {
            return Err("Id is not in logged_out vec")
        }
        Ok(())
    }
    fn enter_chatroom(&mut self,id: u8, id_cr: u8) -> Result<(), &str> {
        let stream = Arc::clone(self.logged_in.get(&id).unwrap());
        match self.chatrooms.get(&id_cr) {
            Some(chatroom) =>  chatroom.lock().unwrap().add_participant(id, stream),
            None => {
                let mut participants: Vec<(u8, Arc<Mutex<TcpStream>>)> = Vec::new();
                participants.push((id, stream));
                self.create_chatroom(participants);
                let new_chatroom_id = self.chatrooms.keys().max().unwrap();
                let new_chatroom = self.chatrooms.get(new_chatroom_id).unwrap();
                let msg = format!("New chatroom with id = {} was created", new_chatroom_id);
                new_chatroom.lock().unwrap().broadcast_message(msg.as_bytes(), id);
            }
        }
        Ok(())
    }
    fn leave_chatroom(&mut self, id: u8, id_cr: u8) -> Result<(), &str> {
        self.chatrooms.get(&id_cr).unwrap().lock().unwrap().remove_participant(id);
        Ok(())
    }
    fn create_chatroom(&mut self, participants: Vec<(u8, Arc<Mutex<TcpStream>>)>) {
        //con overflow...!
        let max_id = match self.chatrooms.keys().max() {
            Some(x) => x + 1,
            None => 0,
        };
        self.chatrooms.insert(max_id, Arc::new(Mutex::new(Chatroom::new(max_id))));
        
        for (id, stream) in participants {
            self.chatrooms.get(&max_id).unwrap().lock().unwrap().add_participant(id, stream);
        }
    }

}