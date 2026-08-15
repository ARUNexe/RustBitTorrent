use crate::{bdecoder::BValue, torrentmeta::TorrentMeta};
use sha1::{Sha1, Digest};


use std::fs;
mod bdecoder;
mod torrentmeta;


fn main() {

    let _test_value: &[u8] = b"d4:name5:Aruns3:agei24e5:peersl6:peer01i6881e6:peer02i6882ee4:infod4:name4:test6:lengthi1024e6:piecesl20:abcdefghijklmnopqrst20:uvwxyzabcdefghijklmneeee";
    let torrent_path = "test_torrent_files/ubuntu_26_04_amd64.torrent";
    let contents = match fs::read(torrent_path){
        Ok(data) => data,
        Err(err) => {
            println!("Unable to open file : {}",err);
            panic!();
        }
    };


    let mut pos: usize = 0;
    let bvalue: BValue = BValue::serialize(&contents, &mut pos).expect("serialize method returned error");
    // bvalue.printer();
    let torrentmeta1 = TorrentMeta::serialize(&bvalue).expect("msg");
    // torrentmeta1.printer();

    // let encoded_bvalue = bvalue.encode().expect("Encoder error");

    // println!("Info Hash = {:x?}",torrentmeta1.info_hash);

    let mut hasher = Sha1::new();
    hasher.update(torrentmeta1.info_hash);
    let res = hasher.finalize();
    println!("SHA-1: {:x}", res);

    // torrent.printer();

}
