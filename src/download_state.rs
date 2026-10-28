use serde::{Serialize, Deserialize};
use std::{fs::OpenOptions, io::{Read, Seek}, path::Path};
use std::io::SeekFrom;

use crate::utils;



#[derive(Serialize, Deserialize)]
#[derive(PartialEq, Eq)]
pub enum PieceStatus {
    Pending,
    Downloading,
    Completed,
}
#[derive(Serialize, Deserialize)]
pub struct Piece{
    pub id: i64,
    pub status: PieceStatus,
}
#[derive(Serialize, Deserialize)]
pub struct DownloadState{
    pub info_hash: Vec<u8>,
    pub piece_count: i64,
    pub piece_length: i64,
    pub pieces: Vec<Piece>,
}

impl DownloadState {
    fn new_pending(nb_pieces: i64, info_hash: Vec<u8>, piece_length: i64) -> Self {
        let pieces = (0..nb_pieces)
            .map(|id| Piece {
                id,
                status: PieceStatus::Pending,
            })
            .collect();

        Self {
            info_hash,
            pieces,
            piece_count: nb_pieces,
            piece_length,
        }
    }

    pub async fn init(nb_pieces: i64,info_hash: Vec<u8>,filename: &str,piece_length: i64) -> Self {


        let filename_json = format!("{}.{}",filename,"json");

        if Path::new(&filename_json).exists() {
            println!("Resuming download from previous state");
            let json = match std::fs::read_to_string(&filename_json) {
                Ok(s) => s,
                Err(_) => {
                    println!("[WARNING] Error reading resume download file");
                    return Self::new_pending(nb_pieces, info_hash, piece_length);
                }
            };
            let mut state: DownloadState = match serde_json::from_str(&json) {
                Ok(s) => s,
                Err(_) => {
                    println!("[WARNING] Error parsing resume download file");
                    return Self::new_pending(nb_pieces, info_hash, piece_length);
                }
            };

            match OpenOptions::new().read(true).write(true).create(true).open(&filename) {
                Ok(mut file) => {
                    for s in state.pieces.iter_mut() {
                        match s.status {
                            PieceStatus::Downloading => { // make all downloading state as pending
                                s.status = PieceStatus::Pending
                            },
                            PieceStatus::Completed => { // verify all the downloaded chunks are with sha1
                                let info_hash_start_idx: usize = (s.id*20) as usize;
                                let info_hash_end_idx = info_hash_start_idx+20 as usize;

                                let file_start_idx;
                                let file_end_idx: i64;
                                let remaining_bytes;

                                if s.id-1 != state.piece_count as i64 {
                                    file_start_idx = s.id*piece_length;
                                    file_end_idx = file_start_idx+piece_length;
                                    remaining_bytes = file_end_idx - file_start_idx;
                                }
                                else {
                                    file_start_idx = s.id*piece_length;
                                    let file_size = match file.metadata() {
                                        Ok(metadata) => metadata.len() as i64,
                                        Err(error) => {
                                            eprintln!("[WARNING] Could not read download file metadata: {error}");
                                            s.status = PieceStatus::Pending;
                                            continue;
                                        }
                                    };
                                    remaining_bytes = file_size.saturating_sub(file_start_idx);
                                }
                                let mut current_piece_data = vec![0u8; remaining_bytes as usize];
                                if let Err(error) = file.seek(SeekFrom::Start(file_start_idx as u64)) {
                                    eprintln!("[WARNING] Could not seek to resume piece {}: {error}", s.id);
                                    s.status = PieceStatus::Pending;
                                    continue;
                                }
                                if let Err(error) = file.read_exact(&mut current_piece_data) {
                                    eprintln!("[WARNING] Could not read resume piece {}: {error}", s.id);
                                    s.status = PieceStatus::Pending;
                                    continue;
                                }

                                let Some(current_piece_verification_hash) = state.info_hash.get(info_hash_start_idx..info_hash_end_idx) else {
                                    eprintln!("[WARNING] No verification hash found for piece {}", s.id);
                                    s.status = PieceStatus::Pending;
                                    continue;
                                };
                                let current_piece_hash = utils::get_sha1(&current_piece_data);

                                if current_piece_verification_hash != current_piece_hash {
                                    s.status = PieceStatus::Pending;
                                }
                            }
                            _ => {
                                
                            }
                        }
                    }
                    state.store_download_state_in_json(&filename);

                    return state;
                },
                Err(_) => {
                    println!("[WARNING] Error opening resume download file");
                }
            };
        }
        else {
            println!("New downloading starting");
        }   

        let ds = Self::new_pending(nb_pieces, info_hash, piece_length);
        ds.store_download_state_in_json(&filename);
        ds
    }

    pub fn get_pending_piece(&self,tried_pieces: &Vec<u8>) ->  i64 {
        for i in 0..self.piece_count {
            if self.pieces[i as usize].status == PieceStatus::Pending && (tried_pieces[i as usize] != 1){
                return self.pieces[i as usize].id;
            }
        }
        -1
    }

    pub fn check_if_all_downloaded(&self) ->  bool {
        for i in 0..self.piece_count {
            if self.pieces[i as usize].status != PieceStatus::Completed{
                return false;
            }
        }
        return true;
    }


    pub fn store_download_state_in_json(&self,filename: &str) {
        let json = match serde_json::to_string_pretty(&self) {
            Ok(json) => json,
            Err(error) => {
                eprintln!("[WARNING] Could not serialize download state: {error}");
                return;
            }
        };
        let filename_json = format!("{}.{}",filename,"json");
        if let Err(error) = std::fs::write(filename_json, json) {
            eprintln!("[WARNING] Could not write download state: {error}");
        }
    }
}

