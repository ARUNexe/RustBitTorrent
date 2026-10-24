use reqwest::Client;
use std::io::{self, Error};

use tokio::{net::UdpSocket};
use std::{net::SocketAddr};
use tokio::net::lookup_host;


use crate::torrentmeta::TorrentMeta;
use crate::peer::PeerInfo;
use crate::utils;
use crate::bencoder::BValue;


async fn get_peer_info_from_http_tracker(torrent: &TorrentMeta,peer_id: String,tracker_url : String) -> Result<PeerInfo,io::Error> {    
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
        tracker_url,
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

    let peerinfo = match PeerInfo::create_from_bvalue(&result_bvalue) {
        Ok(peerinfo) => peerinfo,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Peer info parsing failed")),
    };
    Ok(peerinfo)
}



async fn get_peer_info_from_udp_tracker(torrent: &TorrentMeta,peer_id: String,tracker_url : String) -> Result<PeerInfo,io::Error> {   

    // let sock = UdpSocket::bind("0.0.0.0:6882").await?;

    // let remote_addr = torrent.announce_url;

    // let remote_addr = "udp://tracker.opentrackr.org:1337/announce";

    let address = tracker_url.split('/');
    for i in address {
        println!("address : {i}");
    }
    // let host = address.nth()



    let tracker = "tracker.opentrackr.org:1337";
    println!("Trying connection to udp tracker {}",tracker);

    let addr: SocketAddr = lookup_host(tracker)
        .await?
        .next()
        .ok_or("No tracker address found").expect("Error lookinup host");

    let sock = UdpSocket::bind("0.0.0.0:6882").await.expect("msg");

    match addr {
        SocketAddr::V4(_) => {println!("Reveived ipv4 tracker");},
        SocketAddr::V6(_) => {println!("Reveived ipv6 tracker");}, 
    }

    sock.connect(addr).await?;

    // CONNECT SEQUENCE

    let protocol_id: u64 = 0x41727101980;
    let bytes = protocol_id.to_be_bytes();
    let transaction_id: u32 = 67;
    
    let mut message = Vec::with_capacity(16);
    message.extend_from_slice(&bytes);
    message.extend_from_slice(&0u32.to_be_bytes());
    message.extend_from_slice(&transaction_id.to_be_bytes());


    let _len = sock.send(&message).await?;

    let mut buf = [0; 2048];
    let len = sock.recv(&mut buf).await?;
    println!("{:?} bytes received from {:?}", len, tracker);

    let action = i32::from_be_bytes(buf[0..4].try_into().unwrap());
    let trans_id = i32::from_be_bytes(buf[4..8].try_into().unwrap());
    let connection_id = i64::from_be_bytes(buf[8..16].try_into().unwrap());
    println!("{:?} bytes received", len);

    println!("[CONNECT] Received action : {}, received transaction_id : {}, received connection_id: {}",action,trans_id,connection_id);


    // ANOUNCE SSEQUENCE


    let action: i32 = 1;
    let info_hash_sha1 = utils::get_sha1(&torrent.info_hash);
    let uploaded: i64 = 0;
    let downloaded: i64 = 0;
    let left: i64 = torrent.info.length;
    let event: i32 = 0;
    let ip_addr: i32 = 0; // default. it will take the request's ip ig
    let unique_key: i32 = 67;
    let num_wanted: i32 = -1;
    let port: i32 = 6882;


    let mut anounce_message = Vec::with_capacity(98);
    anounce_message.extend_from_slice(&connection_id.to_be_bytes());
    anounce_message.extend_from_slice(&action.to_be_bytes());
    anounce_message.extend_from_slice(&transaction_id.to_be_bytes());
    anounce_message.extend_from_slice(&info_hash_sha1);
    anounce_message.extend_from_slice(&peer_id.into_bytes());
    anounce_message.extend_from_slice(&downloaded.to_be_bytes());
    anounce_message.extend_from_slice(&left.to_be_bytes());
    anounce_message.extend_from_slice(&uploaded.to_be_bytes());
    anounce_message.extend_from_slice(&event.to_be_bytes());
    anounce_message.extend_from_slice(&event.to_be_bytes());
    anounce_message.extend_from_slice(&ip_addr.to_be_bytes());
    anounce_message.extend_from_slice(&unique_key.to_be_bytes());
    anounce_message.extend_from_slice(&num_wanted.to_be_bytes());
    anounce_message.extend_from_slice(&port.to_be_bytes());



     let _len = sock.send(&anounce_message).await?;


    let mut buf = [0; 1024];
    let len = sock.recv(&mut buf).await?;

    let action = i32::from_be_bytes(buf[0..4].try_into().unwrap());
    let trans_id = i32::from_be_bytes(buf[4..8].try_into().unwrap());
    let intervel = i32::from_be_bytes(buf[8..12].try_into().unwrap());
    let leech = i32::from_be_bytes(buf[12..16].try_into().unwrap());
    let seeders = i32::from_be_bytes(buf[16..20].try_into().unwrap());

    // println!("[ANOUNCE] Received action : {}, received transaction_id : {}, received intervel: {}, leechers : {}, seeder: {}",action,trans_id,intervel,leech,seeders);


    for i in 0..seeders {
        let ip_offset = 20 + (4*i);
        let port_offset = 24 + (2*i);
        println!("IP: {:?} , PORT: {:?}",&buf[ip_offset as usize..(ip_offset+4) as usize],&buf[port_offset as usize..(port_offset+2) as usize]);
    }

    let peer_info = PeerInfo::create_from_udp_bytes(&buf[20..len]);

    match peer_info {
        Ok(pi) => {
            return  Ok(pi);
        },
        Err(_) => {
            println!("Error getting peerInfo from udp tracker");
            return  Err(Error::new(io::ErrorKind::AddrNotAvailable, "fuckup error"));
        }
    }
}

pub async fn get_peer_info_from_tracker(torrent: &TorrentMeta,peer_id: String) -> Result<PeerInfo, io::Error> {
    
    // try main anounce url
    let mut peer_info;
    if torrent.announce_url.contains("http")  {
        peer_info = get_peer_info_from_http_tracker(torrent,peer_id.clone(),torrent.announce_url.clone()).await;
        match peer_info {
            Ok(_) => {return  peer_info;}
            Err(_) => {
                println!("[Warning] Http Tracker in anounce url request failed");
            }
        }
    }
    else if torrent.announce_url.contains("udp") {
        peer_info = get_peer_info_from_udp_tracker(torrent,peer_id.clone(),torrent.announce_url.clone()).await;
        match peer_info {
            Ok(_) => {return  peer_info;}
            Err(_) => {
                println!("[Warning] UDP tracker in anounce url request failed");
            }
        }
    }

    // try all urls in anounce-list if anounce-list exists

    for trackers in torrent.announce_list.iter() {
        let tracker_url;
        match str::from_utf8(&trackers) {
            Ok(url) => {tracker_url = url},
            Err(_) => {continue;},
        }

        if tracker_url.contains("http")  {
            peer_info = get_peer_info_from_http_tracker(torrent,peer_id.clone(),tracker_url.to_string()).await;
            match peer_info {
                Ok(_) => {return  peer_info;}
                Err(_) => {
                    println!("[Warning] Http Tracker from anounce-list request failed");
                }
            }
        }
        else if tracker_url.contains("udp") {
            peer_info = get_peer_info_from_udp_tracker(torrent,peer_id.clone(),tracker_url.to_string()).await;
            match peer_info {
                Ok(_) => {return  peer_info;}
                Err(_) => {
                    println!("[Warning] UDP Tracker from anounce-list request failed");
                }
            }
        }        
    }

   return Err(io::Error::new(io::ErrorKind::NotConnected, "[ERROR] Communication to tracker was not established"));

}