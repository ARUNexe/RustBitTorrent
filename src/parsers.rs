use crate::BValue;
use std::{io, net::{IpAddr, Ipv4Addr}, vec};
use std::net::Ipv6Addr;



#[derive(Default)]
pub struct PeerInfo {
    pub interval: i64,
    pub complete: i64,
    pub incomplete: i64,
    pub peers: Vec<Peer>,
}

pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
    pub peer_id: Vec<u8>,
}


impl PeerInfo {
    pub fn create(bvalue: &BValue) -> Result<Self, io::Error>{
            let mut peer_info =  PeerInfo::default();
            match bvalue {
                BValue::Dicts(dict) => {
                    for entries in dict{
                        let key = entries.0.clone();
                        let value = &entries.1;
                        let key_string = match str::from_utf8(&key) {
                            Ok(strs) => strs,
                            Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing anounce url string")),
                        };
                        match key_string {
                            "complete" => {
                                peer_info.complete = value.get_number()?;
                                println!("Peer info complete is : {}",peer_info.complete);
                            },
                            "incomplete" => {
                                peer_info.incomplete = value.get_number()?;
                                println!("Peer info incomplete is : {}",peer_info.incomplete);
                            },
                            "interval" => {
                                peer_info.interval = value.get_number()?;
                                println!("Peer info interval is : {}",peer_info.interval);
                            },
                            "peers" => {
                                let mut peers: Vec<Peer> = Vec::new();
                                match value {
                                    BValue::Lists(info_value) => {
                                        for entries in info_value{
                                            match entries {
                                                BValue::Dicts(info_value) => {
                                                    for info_entries in info_value.iter(){
                                                        let info_key = info_entries.0.clone();
                                                        let info_value = &info_entries.1;
                                                        let info_key =  match str::from_utf8(&info_key) {
                                                            Ok(strs) => strs,
                                                            Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing info key string")),
                                                        };

                                                        let mut peer = Peer {
                                                            ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                                                            port: 0,
                                                            peer_id: vec![0],
                                                        };
                                                        match info_key {
                                                            "ip" => {
                                                                let bytes = match info_value.get_text() {
                                                                    Ok(bytes) => bytes,
                                                                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Peer IP address is not text")),
                                                                };
                                                                let ipaddr: IpAddr = match bytes.parse() {
                                                                    Ok(ipaddr) => ipaddr,
                                                                    Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Peer IP address parsing failed")),
                                                                };
                                                                peer.ip = ipaddr;
                                                            },
                                                            "port" => {
                                                                peer.port = info_value.get_number()? as u16;
                                                                println!("Port is {}",peer.port);
                                                            },
                                                            "peer id" => {
                                                                peer.peer_id = info_value.get_bytes()?;
                                                                println!("Peer id is : {:?}",peer.peer_id);
                                                            },
                                                            _ => println!("Unsupported key in info_hash"), 
                                                        }
                                                        peers.push(peer);
                                                    }
                                                },
                                                _ => println!("Unsupported key in info_hash in peer list hashing"),
                                            }
                                        }
                                    }
                                    _ => {
                                        println!("Error parsing torrent info file !!");
                                        return Err(io::Error::new(io::ErrorKind::Unsupported, "Info torrent file is not a dict"));
                                    }
                                };
                                peer_info.peers = peers;
                            }
                            _ => println!("Unhandled param, non mandatory"),
                        }
                    }
                }
                _ => {
                    println!("Error parsing torrent file !!");
                    return Err(io::Error::new(io::ErrorKind::Unsupported, "Top torrent file is not a dict"));
                },
            };
            Ok(peer_info)
        }
}