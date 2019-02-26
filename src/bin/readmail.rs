use log::error;
use rms::readmail::extract_body;
mod readmail_cmd;
use mailparse::*;
use readmail_cmd::source;
use std::io::BufRead;

fn main() {
    let src: Box<BufRead> = source();
    let b_msg_rslt = src.split(3);
    for m in b_msg_rslt {
        match m {
            Ok(buf) => {
                if let Ok(msg) = parse_mail(buf.as_slice()) {
                    let headers = &msg.headers;
                    let mut subject: String = "".to_string();
                    let mut from: String = "".to_string();
                    let mut recipients: Vec<String> = vec![];
                    for h in headers {
                        if let Ok(s) = h.get_key() {
                            match s.as_ref() {
                                "Subject" => subject = h.get_value().unwrap_or("".to_string()),
                                "From" => from = h.get_value().unwrap_or("".to_string()),
                                "To" => recipients.push(h.get_value().unwrap_or("".to_string())),
                                "cc" => recipients.push(h.get_value().unwrap_or("".to_string())),
                                "bcc" => recipients.push(h.get_value().unwrap_or("".to_string())),
                                _ => {}
                            }
                        }
                    }

                    println!("From: {}", from);
                    println!("To: {}", recipients.join(", "));
                    println!("Subject: {}", subject);
                    let body = extract_body(msg, true);
                    match body {
                        Some(b) => println!("\n\n{}", b.value),
                        None => {}
                    };
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
