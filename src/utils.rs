use sha1::{Sha1, Digest};



pub fn get_sha1_info_hash(info: &Vec<u8>) ->Vec<u8> {

    let mut hasher = Sha1::new();
    hasher.update(info);
    let info_hash_sha1: sha1::digest::generic_array::GenericArray<u8, sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UInt<sha1::digest::typenum::UTerm, sha1::digest::consts::B1>, sha1::digest::consts::B0>, sha1::digest::consts::B1>, sha1::digest::consts::B0>, sha1::digest::consts::B0>> = hasher.finalize();
    info_hash_sha1.to_vec()
}

pub fn percent_encode(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("%{:02X}", b))
        .collect()
}


pub fn is_peer_handshake_successfull(buffer: &[u8], info_hash: &Vec<u8>) -> bool {
    let protocol_str = "BitTorrent protocol"; 
    
    let message_from_peer = str::from_utf8(&buffer[1..20]).expect("peer message to string fucked");
    if message_from_peer != protocol_str {
        println!("Peer handshake protocol string mismatch");
        return false;
    }

    let peer_info_hash = &buffer[28..48].to_vec();
    // println!("Received peer info hash = {:?}",peer_info_hash);
    // println!("Our peer info hash = {:?}",info_hash);

    if peer_info_hash != info_hash{
        println!("Peer handshake info hash mismatch");
        return false;
    }
    
    return true;
}


