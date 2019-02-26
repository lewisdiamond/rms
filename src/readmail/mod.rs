use html2text::from_read;
use log::{debug, error};
use mailparse::*;
use std::cmp::Ordering;

#[derive(Debug, Default)]
pub struct Body {
    pub mime: String,
    pub value: String,
}
impl Body {
    fn new(mime: String, value: String) -> Body {
        Body { mime, value }
    }
}

fn cmp_body(x: &Option<Body>, y: &Option<Body>, prefer_html: bool) -> Ordering {
    if x.is_none() && y.is_none() {
        return Ordering::Equal;
    }
    if x.is_some() && y.is_none() {
        return Ordering::Less;
    }
    if x.is_none() && y.is_some() {
        return Ordering::Greater;
    }
    let x = x.as_ref().unwrap();
    let y = y.as_ref().unwrap();
    if x.mime == y.mime {
        return Ordering::Equal;
    }
    match x.mime.as_str() {
        "text/body" => Ordering::Greater,
        "text/html" => {
            if prefer_html || y.mime == "text/body" {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        "text/plain" => {
            if !prefer_html || y.mime == "text/body" {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        _ => Ordering::Less,
    }
}
pub fn extract_body(msg: ParsedMail, prefer_html: bool) -> Option<Body> {
    let mut raw_body = None;
    if let Ok(mut text) = msg.get_body() {
        if msg.ctype.mimetype == "text/html" {
            text = from_read(text.as_bytes(), 80);
        }
        raw_body = Some(Body {
            mime: String::from("text/body"),
            value: text,
        });
    };
    let mut bodies: Vec<Option<Body>> = msg
        .subparts
        .into_iter()
        .map(|s| {
            let mime = &s.ctype.mimetype;
            match mime.as_str() {
                "text/plain" => {
                    if let Ok(s) = s.get_body() {
                        Some(Body {
                            mime: mime.clone(),
                            value: String::from(s),
                        })
                    } else {
                        None
                    }
                }
                "text/html" => {
                    if let Ok(s) = s.get_body() {
                        Some(Body {
                            mime: mime.clone(),
                            value: from_read(s.as_bytes(), 80),
                        })
                    } else {
                        None
                    }
                }
                "multipart/alternative" | "multipart/related" => extract_body(s, prefer_html),
                _ => {
                    debug!("unknown mime {}", mime);
                    None
                }
            }
        })
        .collect();

    bodies.push(raw_body);
    bodies.sort_unstable_by(|x, y| cmp_body(x, y, prefer_html));
    bodies.into_iter().filter_map(|x| x).next()
}
