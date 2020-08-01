extern crate select;
use crate::message::{get_id, Body, Message, Mime};
use log::debug;
use mailparse::*;
use select::document::Document;
use select::predicate::Text;
use std::cmp::Ordering;

fn cmp_body(x: &Body, y: &Body, prefer: &Mime) -> Ordering {
    if x.mime == y.mime {
        return x.value.len().cmp(&y.value.len());
    }
    if x.mime == *prefer {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

pub fn extract_body(msg: &ParsedMail, prefer_html: bool) -> Vec<Body> {
    let prefered_mime = if prefer_html {
        Mime::Html
    } else {
        Mime::PlainText
    };
    let text = msg
        .get_body()
        .unwrap_or(msg.get_body_raw().map_or(String::from(""), |x| {
            String::from_utf8(x).unwrap_or(String::from(""))
        }));
    let mime = Mime::from_str(&msg.ctype.mimetype);
    let raw_body = Some(Body::new(mime, text));

    let mut bodies = msg
        .subparts
        .iter()
        .map(|mut s| {
            let mime = Mime::from_str(&s.ctype.mimetype);
            match mime {
                Mime::PlainText | Mime::Html => {
                    s.get_body().ok().map(|b| Body::new(mime, String::from(b)))
                }
                Mime::Nested => extract_body(&mut s, prefer_html).into_iter().next(),
                Mime::Unknown => {
                    debug!("unknown mime {}", mime.as_str());
                    None
                }
            }
        })
        .filter_map(|s| s)
        .collect::<Vec<Body>>();
    if raw_body.is_some() {
        bodies.push(raw_body.expect("COULD NOT UNWRAP RAW_BODY"));
    }
    bodies.sort_unstable_by(|x, y| cmp_body(x, y, &prefered_mime));
    if bodies.len() == 0 {
        println!(
            "No body for message: {}",
            msg.headers
                .iter()
                .map(|x| format!("{}:{}", x.get_key(), x.get_value()))
                .collect::<Vec<String>>()
                .join("\n")
        );
    }
    bodies
}

pub fn html2text(text: &str) -> String {
    let document = Document::from(text);
    let text_nodes = document
        .find(Text)
        .map(|x| x.text())
        .collect::<Vec<String>>();
    return text_nodes.join("\n\n");
}
