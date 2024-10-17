pub enum MessageType {
    LOGIN,
    LOGOUT,
    CHATMESSAGE,
}
pub fn format_msg(msgtype: MessageType, username: &str, content: &str) -> String {
    let mut msg = match msgtype {
        MessageType::LOGIN => String::from("100"),
        MessageType::LOGOUT => String::from("101"),
        MessageType::CHATMESSAGE => String::from("200"),
    };
    msg.push_str(username);
    msg.push_str(content);
    msg.push_str("\n");
    return msg
    
} 