use log::error;
use rms::message::Message;
use rms::readmail::html2text;
mod readmail_cmd;
use readmail_cmd::source;
use rms::message::Mime;
use std::io::BufRead;

fn main() {
    let src: Box<dyn BufRead> = source();
    let b_msg_rslt = src.split(3);
    for m in b_msg_rslt {
        match m {
            Ok(buf) => {
                let message = Message::from_data(buf);
                match message {
                    Ok(message) => {
                        println!("From: {}", message.from);
                        println!("To: {}", message.recipients.join(", "));
                        println!("Subject: {}", message.subject);
                        for b in message.body {
                            println!("Body Mime: {}", b.mime.as_str());
                            match b.mime {
                                Mime::PlainText => println!("\n\n{}", b.value),
                                Mime::Html => println!("\n\n{}", html2text(&b.value)),
                                _ => println!("Unknown mime type"),
                            }
                        }
                    }
                    Err(_e) => error!("Failed to make sense of the message"),
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}
