use reqwest::Client;
use std::io;


use crate::TorrentMeta;
use crate::PeerInfo;
use crate::utils;
use crate::bencoder::BValue;


pub async fn get_peer_info_from_tracker(torrent: &TorrentMeta,peer_id: String) -> Result<PeerInfo,io::Error> {    
    // info hash creation
    let info_hash_sha1 = utils::get_sha1(&torrent.info_hash);

    // tracker params initialization
    let port: i32 = 6882;
    let uploaded = 0;
    let downloaded = 0;
    let left = torrent.info.length;
    let encoded_info_hash = utils::percent_encode(&info_hash_sha1);
    
    // Requesitng the tracker
    let client = Client::new();
    let url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}",
        torrent.announce_url,
        encoded_info_hash,
        peer_id,
        port,
        uploaded,
        downloaded,
        left,
        );

    let request = match client
        .get(&url)
        .build() {
            Ok(req) => req,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Tracker Request building failed")),   
        };

    println!("Final Request is {}",request.url());

    let response = match client.execute(request).await {
        Ok(response) => response,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Tracker request failed")),
    };
    let response_bytes = match response.bytes().await {
        Ok(response_bytes) => response_bytes,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Reading tracker response failed")),
    };

    let mut position = 0;
    let result_bvalue = match BValue::decode(&response_bytes, &mut position) {
        Ok(result_bvalue) => result_bvalue,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Tracker response bencode parsing failed")),
    };

    result_bvalue.printer();

    let peerinfo = match PeerInfo::create(&result_bvalue) {
        Ok(peerinfo) => peerinfo,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Peer info parsing failed")),
    };
    Ok(peerinfo)
}