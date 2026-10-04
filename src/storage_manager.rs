use std::io;
// use tokio::fs;
use std::sync::{Arc,Mutex};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncSeekExt;
use tokio::sync::mpsc::Receiver;
use std::io::SeekFrom;




use crate::download_state;





pub struct CompletedPiece {
    pub index : i64,
    pub data : Vec<u8>,
}

pub struct StorageManager {
    pub file: File,
    pub piece_length: u64,
    pub shared_state : Arc<Mutex<download_state::DownloadState>>,
}


impl StorageManager {
    pub async fn init(filename: String,piece_length: u64,shared_state : Arc<Mutex<download_state::DownloadState>>) -> Result<Self,std::io::Error> {
        let mut file = match File::create(filename).await {
            Ok(f) => f,
            Err(_) => {
                println!("Error opening output file");
                return Err(io::Error::new(io::ErrorKind::InvalidFilename, "Error opening file"));
            }
        };


        return Ok(StorageManager{
            file: file,
            piece_length: piece_length,
            shared_state: shared_state,
        });
    }
}


pub async fn storagemanager_listener_loop(mut storage_manager: StorageManager, mut receiver: Receiver<CompletedPiece>) {

    // println!("Listener Loop Running");
    while let Some(piece) = receiver.recv().await { 
        // println!("Received piece from a peer: Piece index : {}",piece.index);
        write_piece_to_file(&mut storage_manager,piece).await;
    }
    
}

pub async fn write_piece_to_file(storage_manager: &mut StorageManager, completed_piece: CompletedPiece) -> bool {
        
    let file_offset = completed_piece.index as u64 * storage_manager.piece_length as u64;

    match storage_manager.file.seek(SeekFrom::Start(file_offset)).await {
        Ok(_) =>{},
        Err(_) => {return false;},
    }
    match storage_manager.file.write_all(&completed_piece.data).await{
        Ok(_) => {
            // println!("Piece write to the file success Piece indes : {}",completed_piece.index);
        },
        Err(e) => {
            // println!("Error writing piece : {} to file error : {:?}",completed_piece.index,e);
            return false;
        }
    }
    match storage_manager.file.flush().await {
        Ok(_) =>{},
        Err(_) => {return false;},
    }

    let mut ss = storage_manager.shared_state.lock().unwrap();
    // println!("Writing for piece index : {}",completed_piece.index as usize);
    ss.pieces[completed_piece.index as usize].status = download_state::PieceStatus::Completed;
    println!("Piece {} downloaded successfully",completed_piece.index);

    true
}

