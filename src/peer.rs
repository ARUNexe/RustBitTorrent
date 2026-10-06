use crate::BValue;
use std::{io, net::{IpAddr, Ipv4Addr}, vec};


#[derive(Default)]
pub struct PeerInfo {
    pub interval: i64,
    pub complete: i64,
    pub incomplete: i64,
    pub peers: Vec<Peer>,
}

#[derive(Clone)]
pub struct Peer {
    pub ip: IpAddr,
    pub port: u16,
    pub peer_id: Vec<u8>,
    pub peer_bitfield: Vec<u8>,
    pub is_choked: bool,
    pub interested: bool,
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
                            "min interval" => {
                                // peer_info.interval = value.get_number()?;
                                println!("Peer info min interval is : {}",peer_info.interval);
                            },
                            "peers" => {
                                match value {
                                    BValue::Lists(entries) => {
                                        println!("[tracker peers] format=list entries={}", entries.len());
                                    }
                                    BValue::Bytes(bytes) => {
                                        let prefix_len = bytes.len().min(24);
                                        println!(
                                            "[tracker peers] format=byte-string bytes={} compact_ipv4_records={} remainder={} prefix={:02x?}",
                                            bytes.len(),
                                            bytes.len() / 6,
                                            bytes.len() % 6,
                                            &bytes[..prefix_len]
                                        );
                                    }
                                    BValue::Number(_) => println!("[tracker peers] unexpected format=integer"),
                                    BValue::Dicts(_) => println!("[tracker peers] unexpected format=dictionary"),
                                }

                                let mut peers: Vec<Peer> = Vec::new();
                                match value {
                                    BValue::Lists(info_value) => {
                                        for entries in info_value{
                                            match entries {
                                                BValue::Dicts(info_value) => {
                                                    let mut peer = Peer {
                                                        ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                                                        port: 0,
                                                        peer_id: vec![0],
                                                        is_choked: true,
                                                        peer_bitfield: vec![],
                                                        interested: false,
                                                    };
                                                    for info_entries in info_value.iter(){
                                                        let info_key = info_entries.0.clone();
                                                        let info_value = &info_entries.1;
                                                        let info_key =  match str::from_utf8(&info_key) {
                                                            Ok(strs) => strs,
                                                            Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing info key string")),
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
                                                    }
                                                    peers.push(peer);
                                                },
                                                _ => println!("Unsupported key in info_hash in peer list hashing"),
                                            }
                                        }
                                    },
                                    BValue::Bytes(bytes) => {
                                        if bytes.len() % 6 != 0 {
                                            return Err(io::Error::new(
                                                io::ErrorKind::InvalidData,
                                                format!("compact IPv4 peer data has invalid length: {}", bytes.len()),
                                            ));
                                        }

                                        for record in bytes.chunks_exact(6) {
                                            let ip = Ipv4Addr::new(record[0], record[1], record[2], record[3]);
                                            let port = u16::from_be_bytes([record[4], record[5]]);

                                            peers.push(Peer {
                                                ip: IpAddr::V4(ip),
                                                port,
                                                peer_id: Vec::new(),
                                                peer_bitfield: Vec::new(),
                                                is_choked: true,
                                                interested: false,
                                            });
                                        }
                                    },
                                    _ => {
                                        println!("Error parsing torrent info file unknow type in peer");
                                        // return Err(io::Error::new(io::ErrorKind::Unsupported, "Info torrent file is not a dict"));
                                    },
                                };
                                peer_info.peers = peers;
                            }
                            _ => println!("Unhandled param, non mandatory {}",key_string),
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

    pub fn get_peer_at_n(&self,n: usize) -> Peer{
       self.peers[n].clone()
    }
}

