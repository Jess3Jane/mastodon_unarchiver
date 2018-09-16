#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate tar;
extern crate flate2;

use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use flate2::read::GzDecoder;
use tar::Archive;
use serde_json::Value;
use std::env;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Outbox {
    ordered_items: Vec<Toot>,
}

#[derive(Serialize, Deserialize)]
struct Toot {
    actor: String,
    cc: Vec<String>,
    id: String,
    object: Value,
    published: String,
    to: Vec<String>,
    #[serde(rename = "type")]
    t: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TootObject {
    atom_uri: String,
    attributed_to: String,
    cc: Vec<String>,
    content: String,
    conversation: String,
    id: String,
    in_reply_to: Option<String>,
    published: String,
    sensitive: bool,
    to: Vec<String>,
    #[serde(rename = "type")]
    t: String,
    url:String,
}

fn main() {
    let args : Vec<String> = env::args().collect();
    let file = File::open(match args.len() {
        2 => { &args[1] }
        _ => { println!("Usage: {} <archive>", args[0]); return; }
    }).unwrap();
    let g = GzDecoder::new(file);
    let mut a = Archive::new(g);

    for file in a.entries().unwrap() {
        let mut file = file.unwrap();
        if file.header().path().unwrap() != PathBuf::from("outbox.json") { continue; }

        let mut ob: Outbox = serde_json::from_reader(file).unwrap();
        for toot in ob.ordered_items.into_iter()
        .filter(|toot| toot.t == "Create")
        .filter(|toot| 
            toot.cc.contains(&String::from("https://www.w3.org/ns/activitystreams#Public")) ||
            toot.to.contains(&String::from("https://www.w3.org/ns/activitystreams#Public")))
        .filter_map(|toot| match serde_json::from_value::<TootObject>(toot.object) { 
                        Ok(v) => Some(v),
                        Err(_) => None,
                    }) 
        .filter(|toot| !toot.sensitive ) {
            println!("{}", toot.content);
        }
    }
}
