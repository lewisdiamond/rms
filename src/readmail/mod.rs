extern crate select;
use crate::message::{Body, Mime};
use log::debug;
use mailparse::*;
use select::document::Document;
use select::predicate::{Text, Name, Predicate};
use std::cmp::Ordering;

pub mod display;

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
    let text = msg.get_body().unwrap_or_else(|_| {
        msg.get_body_raw().map_or(String::from(""), |x| {
            String::from_utf8(x).unwrap_or_else(|_| String::new())
        })
    });
    let mime = msg.ctype.mimetype.parse::<Mime>().unwrap();
    let raw_body = Some(Body::new(mime, text));

    let mut bodies = msg
        .subparts
        .iter()
        .map(|s| {
            let mime = s.ctype.mimetype.parse::<Mime>().unwrap();
            match mime {
                Mime::PlainText | Mime::Html => s.get_body().ok().map(|b| Body::new(mime, b)),
                Mime::Nested => extract_body(&s, prefer_html).into_iter().next(),
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
    if bodies.is_empty() {
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
    let body = document.find(Name("body")).nth(0).unwrap();
    let text_nodes = body.find(Text)
        .map(|x| String::from(x.text().trim()))
        .filter(|x| x.len() > 1)
        .collect::<Vec<String>>();
    text_nodes.join("\n\n")
}
