use std::collections::HashMap;

pub fn print_messages(messages: &HashMap<String, Vec<String>>) {
    for (file_name, message_list) in messages.iter() {
        println!("Messages for file '{}':", file_name);
        for message in message_list {
            println!("{}", message);
        }
    }
}