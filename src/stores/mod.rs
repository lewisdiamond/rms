use async_trait::async_trait;
use crate::message::Message;

use std::path::PathBuf;
use std::fmt;

pub mod message_store;
pub mod kv;
pub mod search;
pub mod _impl;
pub mod tag;

pub enum MessageStoreError {
    MessageNotFound(String),
    CouldNotAddMessage(String),
    CouldNotOpenMaildir(String),
    CouldNotModifyMessage(String),
    CouldNotDeleteMessage(String),
    CouldNotGetMessage(String),
    CouldNotGetMessages(Vec<String>),
    CouldNotConvertMessage(String),
    InvalidQuery(String),
}

pub trait Store {
    fn add_message(&mut self, msg: Message) -> Result<Message, MessageStoreError>;
    fn delete_message(&mut self, msg: &Message) -> Result<(), MessageStoreError>;
    fn update_message(&mut self, msg: Message) -> Result<Message, MessageStoreError>;
}

impl fmt::Display for MessageStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            MessageStoreError::MessageNotFound(s) => format!("Could not find message {}", s),
            MessageStoreError::CouldNotAddMessage(s) => format!("Could not add message {}", s),
            MessageStoreError::CouldNotOpenMaildir(s) => format!("Could not open maildir {}", s),
            MessageStoreError::CouldNotModifyMessage(s) => {
                format!("Could not modify message {}", s)
            }
            MessageStoreError::CouldNotGetMessages(s) => {
                format!("Could not get messages {}", s.join(", "))
            }
            MessageStoreError::CouldNotGetMessage(s) => format!("Could not get message {}", s),
            MessageStoreError::InvalidQuery(s) => format!("Could query message {}", s),
            MessageStoreError::CouldNotConvertMessage(s) => {
                format!("Could not convert message {}", s)
            },
            MessageStoreError::CouldNotDeleteMessage(s) => format!("Could not delete message {}", s)

        };
        write!(f, "Message Store Error {}", msg)
    }
}


