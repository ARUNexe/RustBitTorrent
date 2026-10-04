use std::io;
use std::ops::ControlFlow::Continue;
use std::sync::Mutex;
use std::sync::Arc;
// use std::sync::mpsc;
use tokio::sync::mpsc;


use tokio::net::tcp::OwnedWriteHalf;
use tokio::stream;
use tokio::{io::AsyncReadExt, net::TcpStream};  
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;

use crate::utils::{is_peer_handshake_successfull};
use crate::download_state::DownloadState;
use crate::peer::Peer;
use crate::utils::get_sha1_info_hash;
use crate::download_state;
use crate::storage_manager::{CompletedPiece};


const BLOCKSIZE: i64 = 16384;

pub async fn send_handshake_to_peer(info_hash: &Vec<u8>,peer_id: &String,stream:&mut TcpStream) -> bool {
    let mut handshake_bytes: Vec<u8> = Vec::with_capacity(68);

    let length: u8 = 19;
    let protocol_str = "BitTorrent protocol"; 
    let protocol: &[u8] = protocol_str.as_bytes();
    let reserved: [u8; 8] = [0; 8]; 

    handshake_bytes.push(length);
    handshake_bytes.extend(protocol);
    handshake_bytes.extend(reserved);
    handshake_bytes.extend(info_hash);
    handshake_bytes.extend(peer_id.bytes());

    
    let res = stream.write_all(&handshake_bytes).await;
    match res {
        Ok(_) =>  {},
        Err(err) => {
            eprintln!("Failed to send handshake: {err}");
            return false;
        }
    }
    let mut buffer = [0u8; 68];
    let res = stream.read_exact(&mut buffer).await;
    match res {
        Ok(_) =>  {},
        Err(err) => {
            eprintln!("Failed to read peer handshake: {err}");
            return false;
        }
    }

    let is_successfull = is_peer_handshake_successfull(&buffer,info_hash);
    is_successfull
}


pub async fn read_peer_message( stream: &mut OwnedReadHalf) -> Result<Vec<u8>, io::Error> {
    // println!("Running read message");
    let mut length_buf = [0u8; 4];

    stream.read_exact(&mut length_buf).await?;

    let length = u32::from_be_bytes(length_buf);

    if length == 0 {
        return Ok(vec![]); // Keep-alive
    }

    // println!("Reading peer message of length : {}",length);

    let mut message = vec![0u8; length as usize];
    stream.read_exact(&mut message).await?;

    // println!("Reading peer message : {:?}",message);

    Ok(message)
}

fn peer_has_piece(bitfield: &[u8], piece_index: usize) -> bool {
    let byte_index = piece_index / 8;
    let bit_index = 7 - (piece_index % 8);
    let mask = 1u8 << bit_index;

    bitfield
        .get(byte_index)
        .is_some_and(|byte| *byte & mask != 0)
}

async fn send_intrested(stream: &mut OwnedWriteHalf) -> std::io::Result<()> {
    let message = [0,0,0,1,2];
    stream.write_all(&message).await?;

    Ok(())
}

async fn send_block_request(stream: &mut OwnedWriteHalf,piece_index: i64,offset: i64, req_size: i32)  -> std::io::Result<()> {


    let piece_index = u32::try_from(piece_index)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "piece index does not fit in u32"))?;
    let offset = u32::try_from(offset)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "offset does not fit in u32"))?;
    let req_size = u32::try_from(req_size)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "request length does not fit in u32"))?;



    let mut message = Vec::with_capacity(17);
    message.extend_from_slice(&13u32.to_be_bytes());
    message.push(6);
    message.extend_from_slice(&piece_index.to_be_bytes());
    message.extend_from_slice(&offset.to_be_bytes());
    message.extend_from_slice(&req_size.to_be_bytes());

    // println!("Sending piece request : {:?}",message);

    stream.write_all(&message).await?;

    Ok(())
}

// async fn received_block_request()


pub async fn reader_loop(mut stream:  OwnedReadHalf, tx: tokio::sync::mpsc::Sender<Vec<u8>>) { 
    loop {

        let result = read_peer_message(&mut stream).await;
        // println!("Message received : {:?}",result);
        match result {
            Ok(bts) => {
                if tx.send(bts).await.is_err() {
                    println!("Error in sending bytes from reader loop");
                    break;
                }
    
            }
            Err(err) => {
                eprintln!("Error reading peer message: {err}");
                break;
            }
        }
    }
}

fn block_size_for_request(
    piece_index: u32,
    offset: u32,
    total_pieces: u32,
    total_size: u64,
    piece_length: u32,
    max_block_size: u32,
) -> Option<u32> {
    if total_pieces == 0 || piece_index >= total_pieces {
        return None;
    }

    let piece_size = if piece_index == total_pieces - 1 {
        let bytes_before_piece = u64::from(piece_index) * u64::from(piece_length);
        u32::try_from(total_size.checked_sub(bytes_before_piece)?).ok()?
    } else {
        piece_length
    };

    let remaining = piece_size.checked_sub(offset)?;
    if remaining == 0 {
        return None;
    }

    Some(remaining.min(max_block_size))
}

pub async fn handle_peer(mut peer: Peer,info_hash: Vec<u8>,my_peerid_c: String,mut stream: TcpStream  ,shared_state: Arc<Mutex<DownloadState>>,piece_length: i64, piece_hash: Vec<u8>,storage_sender: mpsc::Sender<CompletedPiece>,total_data_size: i64) {
    println!("Running handle peer for peer_id {:?}",&peer.peer_id);
    let success = send_handshake_to_peer(&info_hash, &my_peerid_c, &mut stream).await;

    if success {
        println!("My Peer id : {:?} Connected in handshake",&peer.peer_id);
    }
    else {
        println!("My Peer id : {:?} Failed to connect in handshake",&peer.peer_id);
        return;
    }

    let (mut reader, mut writer) = stream.into_split();

    let mut download_intervel = tokio::time::interval(std::time::Duration::from_millis(1000));

    let mut nb_pieces = 0;
    {
        let ss = shared_state.lock().unwrap();
        nb_pieces = ss.nb_piece;
    }
    println!("Total number of pieces  = {}", nb_pieces);

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

    tokio::spawn(reader_loop(reader, tx));


    // Things needs to be reset after one piece is downloaded
    let mut current_piece: i64 = -1;
    // let mut current_piece_bytes_outstanding =  piece_length as i32;
    let mut current_offset = 0;
    // let mut current_piece_data = vec![0u8; piece_length as usize];
    // let mut current_requested_data_size = 0;


    // Things needs to be reset after one block is downloaded
    let mut current_block_req_sent = false;

    let mut pieces_tried = vec![0 as u8; nb_pieces as usize];
    
    loop {
        tokio::select! {
            message = rx.recv() => {
                match message {
                    Some(msg) => {
                        let msg_len = msg.len();
                        // println!("msg length : {}",msg_len);
            
                        if msg[0] == 5 { // bitfield
                            println!("Peer send birfield");
                            peer.peer_bitfield = msg[1..].to_vec();
                        }
                        else if msg[0] == 1 { // unchoke
                            println!("Peer send Unchoke");
                            peer.ischoked = false;
                        }
                        else if msg[0] == 7 { // piece
                            // println!("Piece index received : {:?}",&msg[1..5]);
                            // println!("Piece Offset received : {:?}",&msg[5..9]);


                            let received_piece_index = i32::from_be_bytes(msg[1..5].try_into().unwrap());
                            let received_block_offset = i32::from_be_bytes(msg[5..9].try_into().unwrap());
                            
                            if received_block_offset != current_offset || received_piece_index != current_piece as i32 {
                                println!("Piece index or offset mismatch dropping block");
                                break;
                            }
                            // println!("Got a piece from peer : {:?}",&msg[10..]);
                            // println!("Got a piece from peer");
                            
                            let mut current_piece_data = vec![0u8; (msg.len() - 9) as usize];

                            current_piece_data[received_block_offset as usize..received_block_offset as usize + (msg.len() - 9)].copy_from_slice(&msg[9..]);
                            
                            // println!("Current piece data is  {:?}", current_piece_data);
                            
                            current_offset = current_offset+BLOCKSIZE as i32;
                            current_block_req_sent = false;

                            if current_offset == piece_length as i32 {
                                // println!("LAST BLOCK RECEIVED");

                                let start_idx = (current_piece*20) as usize;
                                let end_idx = start_idx+20 as usize;

                                let current_piece_verification_hash = &piece_hash[start_idx..end_idx].to_vec();
                                println!("Current piece verification hash: {:?}",current_piece_verification_hash);

                                let current_piece_hash = get_sha1_info_hash(&current_piece_data);
                                println!("Current piece data hash = {:?}",current_piece_hash);

                                if current_piece_verification_hash == &current_piece_hash {
                                    

                                    let completed_piece = CompletedPiece {
                                        index : current_piece,
                                        data : current_piece_data.clone(),
                                    };
                                    // println!("Piece hash Verified calling storager");
                                    match storage_sender.send(completed_piece).await {
                                        Ok(_k) => {
                                            // println!("Storage sent successfully : {:?}",k);
                                        },
                                        Err(_e) => {
                                            // println!("Storage sent Failed: {:?}",e);
                                        },
                                    };

                                    current_piece = -1;
                                    current_offset = 0;
                                    current_piece_data = vec![0u8; piece_length as usize];
                                }
                                else {
                                    println!("Piece hash is not same wrong piece downloaded");
                                }
                            }
                        } 
                        else {
                            println!("Peer send unknown msg");
                            println!("Peer send msg {:?}",msg);
                        }
                    },
                    None => {
                        println!("Recevived none in message ");
                        break;
                    }
                }
            }
            
    
            _ = download_intervel.tick() => {

                if peer.peer_bitfield.len() <= 0  {
                    continue;
                }

                if current_piece == -1{
                    let mut ss = shared_state.lock().unwrap();
                    current_piece = ss.get_pending_piece(&pieces_tried);
                    // println!("New unchecked Pending Piece = {}", current_piece);
                    if current_piece == -1 {
                        // pieces_tried[current_piece as usize] = 1;
                        if ss.check_if_all_downloaded() {
                            println!("Download Completed!!!!!");
                        }
                        else{
                            println!("Peer {:?} have no pieces to download",peer.peer_id);
                        }
                        break;
                    }
                
                    let Ok(piece_index) = usize::try_from(current_piece) else {
                        pieces_tried[current_piece as usize] = 1;
                        current_piece = -1;
                        continue;
                    };
                    let is_available = peer_has_piece(&peer.peer_bitfield, piece_index);
                    if is_available {
                        println!("Peer has current piece: {}", is_available);
                        ss.pieces[current_piece as usize].status = download_state::PieceStatus::Downloading;
                    }
                    else {
                        println!("Peer doesnot hava current piece: {}", is_available);
                        pieces_tried[current_piece as usize] = 1;
                        current_piece = -1;
                        continue;
                    }
                }

                if peer.intrested == false {
                    let res = send_intrested(&mut writer).await;
                    match res {
                        Ok(_) => {
                            println!("Intrested message sent successfully");
                            peer.intrested = true;
                        },
                        Err(_) => {
                            println!("Error in sending Intrested message");
                            continue;
                        },
                    }
                }

                if !peer.ischoked{
                    if current_block_req_sent == false
                    {
                        // println!("Peer id : {} :Ready for download",peer.ip);

                        let Some(block_len) = block_size_for_request(current_piece as u32,current_offset as u32,nb_pieces as u32,total_data_size as u64,piece_length as u32,BLOCKSIZE as u32,
                        ) else {
                            continue;
                        };



                        let res = send_block_request(&mut writer,current_piece,current_offset as i64,block_len as i32).await;
                        match res {
                            Ok(_) => {
                                println!("Sent block request for piece {} , offset {} , size {}",current_piece,current_offset,block_len);
                                current_block_req_sent = true;
                            },
                            Err(_) => {
                                println!("Error sending blocl request");
                            }
                            
                        }
                    }
                }
            }
    
        }
    }



    // let res = listen_peer(&mut peer,&info_hash,my_peerid_c,&mut stream).await;
}
