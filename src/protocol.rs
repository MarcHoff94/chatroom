

pub fn format_msg_to_client_protocol(sender_id: String, sender: String, content: String) -> String {
    return format!("{}{}{}", sender_id, sender, content)
}