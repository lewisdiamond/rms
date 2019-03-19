use log::{debug, error};
use rms::message::{get_id, Message};
use rms::readmail::{extract_body, html2text};
mod readmail_cmd;
use mailparse::*;
use readmail_cmd::source;
use rms::message::Mime;
use std::io::BufRead;

fn main() {
    let src: Box<BufRead> = source();
    let b_msg_rslt = src.split(3);
    for m in b_msg_rslt {
        match m {
            Ok(buf) => {
                let hash = get_id(&buf);
                if let Ok(mut msg) = parse_mail(buf.as_slice()) {
                    let message = Message::from_parsedmail(&mut msg, hash);
                    match message {
                        Ok(message) => {
                            println!("From: {}", message.from);
                            println!("To: {}", message.recipients.join(", "));
                            println!("Subject: {}", message.subject);
                            let body = extract_body(&mut msg, false);
                            for b in body {
                                println!("Body Mime: {}", b.mime.as_str());
                                match b.mime {
                                    Mime::PlainText => println!("\n\n{}", b.value),
                                    Mime::Html => println!("\n\n{}", html2text(&b.value)),
                                    _ => println!("Unknown mime type"),
                                }
                            }
                        }
                        Err(e) => error!("Failed to make sense of the message"),
                    }
                } else {
                    error!("Failed to parse the file");
                    ::std::process::exit(1);
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}
