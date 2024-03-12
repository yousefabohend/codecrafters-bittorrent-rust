
use anyhow::Ok;
use serde_json;
use std::{env, path::PathBuf};

// Available if you need it!
use serde_bencode;
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> anyhow::Result<serde_json::Value> {
    // If encoded_value starts with a digit, it's a number
    let value: serde_bencode::value::Value = serde_bencode::from_str(encoded_value)?;
    convert(value)
}
fn convert(value: serde_bencode::value::Value) -> anyhow::Result<serde_json::Value> {
    match value {
        serde_bencode::value::Value::Bytes(b) => 
            Ok(serde_json::Value::String(String::from_utf8(b)?)),
        serde_bencode::value::Value::Int(i) => Ok(serde_json::Value::Number(   serde_json::Number::from(i))),
        serde_bencode::value::Value::List(l) => {
            let mut vec = Vec::new();
            for v in l {
                vec.push(convert(v)?);
            }
            Ok(serde_json::Value::Array(vec))
        }
        serde_bencode::value::Value::Dict(d) => {
            let mut map = serde_json::Map::new();
            for (k, v) in d {
                map.insert(String::from_utf8(k)?, convert(v)?);
            }
            Ok(serde_json::Value::Object(map))
        }
        //todo: handle other cases
        /*_ => {
            panic!("Unreachable encoded value{:?}", value)
        }*/
    }
}

#[derive(Serialize, Deserialize)]
struct TorrentInfo {
    name: String,
    #[serde(rename = "piece length")]
    piece_length: u64,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
    length: usize,
}
#[derive(Serialize, Deserialize)]
struct Torrent {
announce: String,
info: TorrentInfo,
}
fn load_torrent_file<T>(file_path:T) -> anyhow::Result<Torrent> where T: Into<PathBuf> {
    let content = std::fs::read(file_path.into())?;
    let torrent: Torrent = serde_bencode::from_bytes(&content)?;
    Ok(torrent)
}
// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()>{
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        //println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value)?;
        println!("{}", decoded_value.to_string());
    }else if command == "info" {
        let file_path = &args[2];        
        let torrent = load_torrent_file(file_path)?;
        println!("Tracker URL: {}\nLength: {}",torrent.announce,torrent.info.length);
        //print!("Tracker: {}\n", torrent.announce);
     //   println!("Length: {}", torrent.info.length);
    } 
    else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}
