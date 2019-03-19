use crate::message::{Body, Mime};
use html5ever::rcdom::{Handle, Node, NodeData, RcDom};
use html5ever::serialize::{serialize, SerializeOpts};
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{local_name, parse_document, ParseOpts};
use log::{debug, error};
use mailparse::*;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

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

pub fn extract_body(msg: &mut ParsedMail, prefer_html: bool) -> Vec<Body> {
    let mut raw_body = None;
    let prefered_mime = if prefer_html {
        Mime::Html
    } else {
        Mime::PlainText
    };
    if let Ok(text) = msg.get_body() {
        let mime = Mime::from_str(&msg.ctype.mimetype);
        raw_body = Some(Body::new(mime, String::from(text)));
    };
    let mut bodies = msg
        .subparts
        .iter_mut()
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
    bodies
}

pub fn html2text(text: &str) -> String {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut text.as_bytes())
        .expect("COULD NOT UNWRAP DOM");
    let document_children = dom.document.children.borrow();
    let html = document_children.get(0).expect("COULD NOT UNWRAP HTML");
    let body_rc = html.children.borrow();
    let body = body_rc
        .iter()
        .filter(|n| match n.data {
            NodeData::Element { ref name, .. } => name.local == local_name!("body"),
            _ => false,
        })
        .next();

    let ret = match body {
        Some(b) => render_tag(&b)
            .into_iter()
            .filter(|s| s != "")
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>()
            .join("\n"),
        None => "".to_string(),
    };
    ret
}

pub fn render_tag(node: &Handle) -> Vec<String> {
    let mut ret = vec![];
    match node.data {
        NodeData::Text { ref contents } => ret.push(contents.borrow().trim().to_string()),
        _ => {}
    };
    for child in node.children.borrow().iter() {
        ret.append(&mut render_tag(child));
    }
    ret
}
