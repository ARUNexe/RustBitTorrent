use serde::{Serialize, Deserialize};
use std::path::Path;



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
    pub piece_count: u64,
    pub pieces: Vec<Piece>,
}

impl DownloadState {
    pub fn init(nb_pieces: i64,info_hash: Vec<u8>,filename: &str) -> Self {


        let filename_json = format!("{}.{}",filename,"json");

        if Path::new(&filename_json).exists() {
            println!("Resume metadata available");

            let json = std::fs::read_to_string(&filename_json).expect("Failed while loading resume metadata from jsonss");
            let mut state: DownloadState = serde_json::from_str(&json).expect("Failed while loading resume metadata from json");

            // make all downloading state as pending
            for s in state.pieces.iter_mut() {
                match s.status {
                    PieceStatus::Downloading => {
                        s.status = PieceStatus::Pending
                    }
                    _ => {}
                } 
            }

            state.store_download_state_in_json(&filename);

            return state;
        }
        else {
            println!("New downloading starting");
        }   

        let mut vec_pieces:  Vec<Piece> = Vec::new();
        for num in 0..nb_pieces {
            vec_pieces.push(Piece{
                id: num,
                status: PieceStatus::Pending,
            });
        }
        let ds = DownloadState{
            info_hash: info_hash,
            pieces:vec_pieces,
            piece_count: nb_pieces as u64,
        };
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
        let json = serde_json::to_string_pretty(&self).expect("Error in seder");
        let filename_json = format!("{}.{}",filename,"json");
        std::fs::write(filename_json, json).expect("Error in seder");       
    }
}

