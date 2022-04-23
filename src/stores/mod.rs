use crate::message::Message;

use std::fmt;

pub mod _impl;
pub mod kv;
pub mod message_store;
pub mod search;

#[derive(Debug)]
pub enum MessageStoreError {
    MessageNotFound(String),
    CouldNotAddMessage(String),
    CouldNotOpenMaildir(String),
    CouldNotModifyMessage(String),
    CouldNotDeleteMessage(String),
    CouldNotGetMessage(String),
    CouldNotGetMessages(Vec<String>),
    CouldNotConvertMessage(String),
    CouldNotCreateKvError(String),
    CouldNotCreateSearcherError(String),
    FailedToMoveParsedMailEntry(std::io::Error),
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
            }
            MessageStoreError::CouldNotDeleteMessage(s) => {
                format!("Could not delete message {}", s)
            }

            MessageStoreError::CouldNotCreateKvError(_) => {
                format!("Could not create the KV store")
            }
            MessageStoreError::CouldNotCreateSearcherError(_) => {
                format!("Could not create the Search store")
            }
            MessageStoreError::FailedToMoveParsedMailEntry(_) => {
                format!("Could not move parsed mail entry")
            }
        };
        write!(f, "Message Store Error {}", msg)
    }
}
