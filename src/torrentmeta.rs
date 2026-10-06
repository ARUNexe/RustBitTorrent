use std::io;
use crate::bencoder::BValue;

#[derive(Default)]
pub struct InfoHash{
    pub length: i64,
    pub name: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
}

#[derive(Default)]
pub struct TorrentMeta{
    pub announce_url: String,
    pub comment: String,
    pub created_by: String,
    pub creation_date: i64,
    pub info: InfoHash,
    pub info_hash: Vec<u8>,
}

#[allow(dead_code)]
impl TorrentMeta {
        pub fn create(bvalue: &BValue) -> Result<Self, io::Error>{
            let mut torrent_meta = TorrentMeta::default();
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
                            "announce" => {
                                torrent_meta.announce_url = value.get_text()?;
                            },
                            "comment" => {
                                torrent_meta.comment = value.get_text()?;
                            },
                            "created by" => {
                                torrent_meta.created_by = value.get_text()?;
                            },
                            "creation date" => {
                                torrent_meta.creation_date = value.get_number()?;
                            }
                            "info" => {
                                torrent_meta.info_hash = value.encode()?;

                                match value {
                                    BValue::Dicts(info_value) => {
                                        for info_entries in info_value{
                                            let info_key = info_entries.0.clone();
                                            let info_value = &info_entries.1;
                                            let info_key =  match str::from_utf8(&info_key) {
                                                Ok(strs) => strs,
                                                Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Error while paarsing info key string")),
                                            };
                                            match info_key {
                                                "length" => {
                                                    torrent_meta.info.length = info_value.get_number()?;
                                                },
                                                "name" => {
                                                    torrent_meta.info.name = info_value.get_text()?;
                                                },
                                                "piece length" => {
                                                    torrent_meta.info.piece_length = info_value.get_number()?;
                                                },
                                                "pieces" => {
                                                    torrent_meta.info.pieces = info_value.get_bytes()?;
                                                },
                                                _ => println!("Unsupported key in info_hash"),
                                             }
                                        }
                                    }
                                    _ => {
                                        println!("Error parsing torrent info file !!!!!");
                                        return Err(io::Error::new(io::ErrorKind::Unsupported, "Info torrent file is not a dict"));
                                    }
                                };
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
            Ok(torrent_meta)
        }

        pub fn printer(&self){
            // println!("INFO - Pieces : {:02x?}",self.info.pieces);
            println!("INFO - Length : {}",self.info.length);
            println!("INFO - Name : {}",self.info.name);
            println!("INFO - Piece Length : {}",self.info.piece_length);
            // println!("INFO - Pieces len: {:?}",self.info.pieces);

            println!("Anounce URL : {}",self.announce_url);
            println!("Comment : {}",self.comment);
            println!("Created By : {}",self.created_by);
            println!("Creation Date : {}",self.creation_date);

            // println!("Info hash = {:?}",self.info_hash);
        }

}
