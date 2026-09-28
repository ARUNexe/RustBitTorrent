use std::{fs, io};
use rand::distr::{Alphanumeric, SampleString};


mod bencoder;
mod torrentmeta;
mod peer;
mod comms;
mod utils;


use bencoder::BValue;
use torrentmeta::TorrentMeta;

use crate::peer::PeerInfo;
use crate::peer::Peer;
use crate::comms::comms_peer;
use crate::comms::comms_tracker;



#[tokio::main]
async fn main() -> Result<(), io::Error> {

    let torrent_path = "test_torrent_files/ubuntu_26_04_amd64.torrent";
    let contents = match fs::read(torrent_path){
        Ok(data) => data,
        Err(err) => {
            println!("Unable to open file : {}",err);
            panic!();
        }
    };

    let mut pos: usize = 0;
    let bvalue: BValue = match BValue::decode(&contents, &mut pos) {
        Ok(bvalue) => bvalue,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Torrent file bencode parsing failed")),
    };
    let torrentmeta = match TorrentMeta::create(&bvalue) {
        Ok(torrentmeta) => torrentmeta,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Torrent metadata parsing failed")),
    };
    // torrentmeta1.printer();

    let info_hash = utils::get_sha1_info_hash(&torrentmeta.info_hash);

    let peer_id = Alphanumeric.sample_string(&mut rand::rng(), 20);
    let peer_info: PeerInfo = comms_tracker::get_peer_info_from_tracker(&torrentmeta,peer_id.clone()).await?;

    let first_peer: Peer = peer_info.get_peer_at_n(0);

    // println!("Peer ip : {}",first_peer.ip);
    // println!("Peer id : {:?}",first_peer.peer_id);
    // println!("Peer [prt : {}",first_peer.port);

    let is_successfull = comms_peer::send_handshake_to_peer(first_peer,&info_hash, peer_id).await;

    if is_successfull{
        println!("Peer handshake Successfull");
    }

    Ok(())
}
