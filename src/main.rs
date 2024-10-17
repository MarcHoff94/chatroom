use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Server is running on 127.0.0.1:8080");

    let chatroom = Arc::new(Mutex::new(Chatroom::new(1)));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                
                let chatroom_clone = Arc::clone(&chatroom);
                let stream = Arc::new(Mutex::new(stream));

                thread::spawn(move || {
                    handle_client(chatroom_clone, stream);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }

    Ok(())  
}

fn handle_client(chatroom: Arc<Mutex<Chatroom>>, stream: Arc<Mutex<TcpStream>>) {
    let mut reader = BufReader::new(stream.lock().unwrap().try_clone().unwrap());

    loop {
        let mut buffer = String::new();

        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                //chatroom.lock().unwrap().remove_participant(username);
                break;
            }
            Ok(_) => {
                let mut raw_msg = buffer.split(";");
                let msgtype = raw_msg.next().unwrap();
                let username = String::from(raw_msg.next().unwrap());
                let content = String::from(raw_msg.next().unwrap_or_default().trim());
                println!("Client {} sent: {}", username, content);
                match msgtype {
                    "100" => chatroom.lock().unwrap().add_participant(username, Arc::clone(&stream)),
                    "200" => chatroom.lock().unwrap().broadcast_message(content,username),
                    _ => chatroom.lock().unwrap().broadcast_message(content,username)
                }
                
            }
            Err(e) => {
                println!("Error reading from client: {}", e);
                //chatroom.lock().unwrap().remove_participant(username);
                break;
            }
        }
    }
}

struct Chatroom {
    id: u8,
    participants: HashMap<String, Arc<Mutex<TcpStream>>>,
}

impl Chatroom {
    fn new(id: u8) -> Chatroom {
        Chatroom {
            id,
            participants: HashMap::new(),
        }
    }

    fn broadcast_message(&self, message: String, sender: String) {
        let msg = format!("{}: {}", sender, message);
        for (username, participant) in &self.participants {
            if *username != sender {
                let mut stream = participant.lock().unwrap();
                if let Err(e) = stream.write_all(msg.as_bytes()) {
                    println!("Error sending message to client {}: {}", username, e);
                }
            }
        }
    }
    fn add_participant(&mut self, username: String, stream: Arc<Mutex<TcpStream>>) {
        self.participants.insert(username.clone(), stream);
        let msg = format!("{} has entered the chatroom", username);
        self.broadcast_message(msg, username);
    }
    fn remove_participant(&mut self, username: String) {
        self.participants.remove(&username);
        let msg = format!("Removed {} from chatroom", username);
        self.broadcast_message(msg, username);
    }
}

struct Lobby {
    logged_in: HashMap<String, Arc<Mutex<TcpStream>>>,
    logged_out: Vec<String>,
    chatrooms: HashMap<u8, Arc<Mutex<Chatroom>>>,
    new_id: u8
}
impl Lobby {
    fn new() -> Lobby {
        Lobby { logged_in: HashMap::new(), logged_out: Vec::new(), chatrooms: HashMap::new(), new_id: 0 }
    }
    fn log_in(&mut self, username: String, stream: Arc<Mutex<TcpStream>>) -> Result<(), &str> {
        self.log_out(&username).unwrap_or_else(|_| {});
        self.logged_in.insert(username, stream);
        Ok(())
    }
    fn log_out(&mut self, username: &String) -> Result<(), &str> {
        if self.logged_out.contains(username) {
            let idx = self.logged_out.binary_search_by(|x| x.cmp(username)).unwrap();
            self.logged_out.remove(idx);
        } else {
            return Err("Id is not in logged_out vec")
        }
        Ok(())
    }
    fn enter_chatroom(&mut self,username: String, id_cr: u8) -> Result<(), &str> {
        let stream = Arc::clone(self.logged_in.get(&username).unwrap());
        match self.chatrooms.get(&id_cr) {
            Some(chatroom) =>  chatroom.lock().unwrap().add_participant(username, stream),
            None => {
                let mut participants: Vec<(String, Arc<Mutex<TcpStream>>)> = Vec::new();
                participants.push((username.clone(), stream));
                self.create_chatroom(participants);
                let new_chatroom_id = self.chatrooms.keys().max().unwrap();
                let new_chatroom = self.chatrooms.get(new_chatroom_id).unwrap();
                let msg = format!("New chatroom with id = {} was created", new_chatroom_id);
                new_chatroom.lock().unwrap().broadcast_message(msg, username);
            }
        }
        Ok(())
    }
    fn leave_chatroom(&mut self, username: String, id_cr: u8) -> Result<(), &str> {
        self.chatrooms.get(&id_cr).unwrap().lock().unwrap().remove_participant(username);
        Ok(())
    }
    fn create_chatroom(&mut self, participants: Vec<(String, Arc<Mutex<TcpStream>>)>) {
        //con overflow...!
        let max_id = match self.chatrooms.keys().max() {
            Some(x) => x + 1,
            None => 0,
        };
        self.chatrooms.insert(max_id, Arc::new(Mutex::new(Chatroom::new(max_id))));
        
        for (username, stream) in participants {
            self.chatrooms.get(&max_id).unwrap().lock().unwrap().add_participant(username, stream);
        }
    }

}