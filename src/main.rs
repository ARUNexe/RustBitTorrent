use std::{fs, io};
use rand::distr::{Alphanumeric, SampleString};
mod bencoder;
mod torrentmeta;
mod communication;
mod parsers;


use bencoder::BValue;
use torrentmeta::TorrentMeta;


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

    let peer_id = Alphanumeric.sample_string(&mut rand::rng(), 20);
    communication::get_peer_info_from_tracker(&torrentmeta,peer_id).await?;
    Ok(())
}
