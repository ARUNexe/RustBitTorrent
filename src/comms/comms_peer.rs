use std::{net::SocketAddr};
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio::io::AsyncWriteExt;

use crate::peer::Peer;
use crate::utils::{is_peer_handshake_successfull};

pub async fn send_handshake_to_peer(peer: Peer,info_hash: &Vec<u8>,peer_id: String) -> bool {
    let mut handshake_bytes: Vec<u8> = Vec::with_capacity(68);

    let length: u8 = 19;
    let protocol_str = "BitTorrent protocol"; 
    let protocol: &[u8] = protocol_str.as_bytes();
    let reserved: [u8; 8] = [0; 8]; 

    handshake_bytes.push(length);
    handshake_bytes.extend(protocol);
    handshake_bytes.extend(reserved);
    handshake_bytes.extend(info_hash);
    handshake_bytes.extend(peer_id.bytes());

    let addr = SocketAddr::new(peer.ip, peer.port);
    println!("The peer ip is {}",addr);

    let mut stream = TcpStream::connect(addr).await.expect("msg");
    
    stream.write_all(&handshake_bytes).await.expect("Sending message Fucked");

    let mut buffer = [0u8; 68];
    stream.read_exact(&mut buffer).await.expect("Awaiting message fucked");

    let is_successfull = is_peer_handshake_successfull(&buffer,info_hash);
    is_successfull
}