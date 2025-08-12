#![allow(dead_code)]

use once_cell::sync::Lazy;
use std::sync::Mutex;

#[macro_export]
macro_rules! log {
    ($level:ident, $fmt:expr, $lb:expr) => {
        $crate::debug::LOG.lock().unwrap().add(
            $crate::debug::MessageType::from_str(stringify!($level)),
            $fmt.to_string(),
            $lb
        );
    };
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType { Info, Note, Debug, Warn, Error, }

impl MessageType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "info" => MessageType::Info,
            "note" => MessageType::Note,
            "debug" => MessageType::Debug,
            "warn" => MessageType::Warn,
            "error" => MessageType::Error,
            _ => MessageType::Info,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    mtype: MessageType,
    content: String,
    line_break: bool,
}

impl Message {
    pub fn new(mtype: MessageType, content: String, line_break: bool) -> Self {
        Message { mtype, content, line_break }
    }
}

pub struct Log {
    messages: Vec<Message>,
}

impl Log {
    pub fn new() -> Self {
        Log { messages: Vec::new() }
    }

    pub fn add(&mut self, message_type: MessageType, content: String, line_break: bool) {
        self.messages.push(Message::new(message_type, content, line_break));
    }

    pub fn print_all(&self) {
        for msg in &self.messages {
            println!("[{:?}] {}", msg.mtype, msg.content);
            if msg.line_break {
                println!();
            }
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

pub static LOG: Lazy<Mutex<Log>> = Lazy::new(|| Mutex::new(Log::new()));
