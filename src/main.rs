use serde_bytes::ByteBuf;
// Available if you need it!
use sha1::{Digest, Sha1};
use serde_bencode;
use serde::{Serialize, Deserialize};
use anyhow::Ok;
use serde_json;
use std::{env, io::{Read, Write}, path::PathBuf};
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
#[derive(Serialize, Deserialize)]
struct TrackerResponse {
    interval: i64,
    peers: ByteBuf,
}
fn load_torrent_file<T>(file_path:T) -> anyhow::Result<Torrent> where T: Into<PathBuf> {
    let content = std::fs::read(file_path.into())?;
    let torrent: Torrent = serde_bencode::from_bytes(&content)?;
    Ok(torrent)
}
fn calculate_info_hash(info: &TorrentInfo) -> anyhow::Result<[u8; 20]> {
    let info = serde_bencode::to_bytes(info)?;
    let mut hasher = Sha1::new();
    hasher.update(info);
    Ok(hasher.finalize().into())
}
fn request_tracker(torrent :&Torrent, hash: &[u8;20] ) -> anyhow::Result<TrackerResponse> {
    let mut hash_encoded = String::with_capacity(60);
    for &byte in hash {
        hash_encoded.push('%');
        hash_encoded.push_str(&hex::encode(&[byte]));
    }
    let url = format!("{}?info_hash={}&peer_id=12345678901234567890&port=6881&uploaded=0&downloaded=0&left={}&compact=1", torrent.announce, hash_encoded, torrent.info.length);
    let response = reqwest::blocking::get(url).unwrap();
    let body = response.bytes()?;
    let response: TrackerResponse = serde_bencode::from_bytes(&body).unwrap();
    Ok(response)
}
fn get_peer_id( hash: &[u8;20], peer: &String) -> anyhow::Result<String>{
    let protocol = "BitTorrent protocol";
    let reserved = [0u8; 8];
    let  peer_id = [0u8; 20];
    let mut  handshake = Vec::new();
    handshake.push(protocol.len() as u8);
    handshake.extend(protocol.as_bytes());
    handshake.extend(&reserved);
    handshake.extend(hash);
    handshake.extend(peer_id);

    let mut stream = std::net::TcpStream::connect(peer)?;
    stream.write_all(&handshake)?;
    let mut buffer = [0; 68];
    stream.read_exact(&mut buffer)?;
    //let protocol_string_length = buffer[0];
    //let protocol_string = &buffer[1..20];
    //let reserved = &buffer[20..28];
    //let info_hash = &buffer[28..48];
    let peer_id = &buffer[48..68];
    Ok(hex::encode(peer_id))
    
 
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
        let torrent = load_torrent_file(&args[2])?;
        let hash = hex::encode(calculate_info_hash(&torrent.info)?);
        println!("Tracker URL: {}", torrent.announce);
        println!("Length: {}", torrent.info.length);
        println!("Info Hash: {}", hash);
        println!("Piece Length: {}", torrent.info.piece_length);
        println!("Piece Hashes: {}", hex::encode(torrent.info.pieces));
    } else if command == "peers"{
        let torrent = load_torrent_file(&args[2])?;
        let hash = calculate_info_hash(&torrent.info)?;
        let response = request_tracker(&torrent, &hash)?;
        response.peers.chunks(6).for_each(|chunk| {
            let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            println!("{}:{}", ip, port);
        });
        
    }
    else if command == "handshake" {
        let torrent = load_torrent_file(&args[2])?;
        let hash = calculate_info_hash(&torrent.info)?;
        let peer_id = get_peer_id(&hash, &args[3])?;
        println!("Peer ID: {}", peer_id);
    }
    else {
        println!("unknown command: {}", args[1])
    }
    Ok(())
}
