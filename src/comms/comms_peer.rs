use std::io;
use std::sync::Mutex;
use std::sync::Arc;
use tokio::net::tcp::{OwnedWriteHalf,OwnedReadHalf};
use tokio::{sync::mpsc,io::AsyncWriteExt, io::AsyncReadExt, net::TcpStream};  


use crate::utils::{is_peer_handshake_successfull,get_sha1};
use crate::peer::Peer;
use crate::download_state;
use crate::storage_manager::{CompletedPiece};


const BLOCK_SIZE: i64 = 16384;

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
    let mut length_buf = [0u8; 4];

    stream.read_exact(&mut length_buf).await?;
    let length = u32::from_be_bytes(length_buf);
    if length == 0 {
        return Ok(vec![]); // Keep-alive
    }
    let mut message = vec![0u8; length as usize];
    stream.read_exact(&mut message).await?;

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

async fn send_interested(stream: &mut OwnedWriteHalf) -> std::io::Result<()> {
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

fn block_size_for_request( piece_index: u32, offset: u32, total_pieces: u32, total_size: u64, piece_length: u32, max_block_size: u32) -> u32 {


    let piece_size = if piece_index == total_pieces - 1 {
        let bytes_before_piece = u64::from(piece_index) * u64::from(piece_length);
        (total_size-bytes_before_piece) as u32
    } else {
        piece_length
    };

    let remaining = piece_size - offset;
  
    remaining.min(max_block_size)
}


pub async fn handle_peer(mut peer: Peer,info_hash: Vec<u8>,my_peerid_c: String,mut stream: TcpStream  ,shared_state: Arc<Mutex<download_state::DownloadState>>,piece_length: i64, piece_hash: Vec<u8>,storage_sender: mpsc::Sender<CompletedPiece>,total_data_size: i64) {
    println!("Running handle peer for peer_id {:?}",&peer.peer_id);
    let success = send_handshake_to_peer(&info_hash, &my_peerid_c, &mut stream).await;

    if success {
        println!("My Peer id : {:?} Connected in handshake",&peer.peer_id);
    }
    else {
        println!("My Peer id : {:?} Failed to connect in handshake",&peer.peer_id);
        return;
    }

    let (reader, mut writer) = stream.into_split();

    let mut download_intervel = tokio::time::interval(std::time::Duration::from_millis(1000));

    let nb_pieces;
    {
        let ss = shared_state.lock().unwrap();
        nb_pieces = ss.piece_count;
    }
    println!("Total number of pieces  = {}", nb_pieces);


    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);
    let reader_task_handle = tokio::spawn(reader_loop(reader, tx));

    // Things needs to be reset after one piece is downloaded
    let mut current_piece: i64 = -1;
    let mut current_offset = 0;
    let mut current_piece_data = vec![0u8; piece_length as usize];
    let mut current_piece_size = piece_length as usize;

    // Things needs to be reset after one block is downloaded
    let mut current_block_req_sent = false;

    let mut pieces_tried = vec![0 as u8; nb_pieces as usize];
    
    loop {
        tokio::select! {
            message = rx.recv() => {
                match message {
                    Some(msg) => {
            
                        if msg[0] == 5 { // bitfield
                            println!("Peer send birfield");
                            peer.peer_bitfield = msg[1..].to_vec();
                        }
                        else if msg[0] == 1 { // unchoke
                            println!("Peer send Unchoke");
                            peer.is_choked = false;
                        }
                        else if msg[0] == 7 { // piece

                            let received_piece_index = i32::from_be_bytes(msg[1..5].try_into().unwrap());
                            let received_block_offset = i32::from_be_bytes(msg[5..9].try_into().unwrap());
                            
                            if received_block_offset != current_offset || received_piece_index != current_piece as i32{
                                println!("Piece index or offset mismatch dropping block");
                                break;
                            }

                            let block = &msg[9..];
                            // println!("Received piece block: piece {received_piece_index}, offset {received_block_offset}, data {} bytes (message {} bytes)", block.len(), msg.len());
                            let start = received_block_offset as usize;
                            let end = start + block.len();
                            current_piece_data[start..end].copy_from_slice(block);
                            
                            current_offset += block.len() as i32;

                            current_block_req_sent = false;

                            if current_offset == current_piece_size as i32 {
                                // println!("LAST BLOCK RECEIVED");

                                let start_idx = (current_piece*20) as usize;
                                let end_idx = start_idx+20 as usize;

                                let current_piece_verification_hash = &piece_hash[start_idx..end_idx].to_vec();
                                let current_piece_hash = get_sha1(&current_piece_data);


                                if current_piece_verification_hash == &current_piece_hash {
                                    
                                    let completed_piece = CompletedPiece {
                                        index : current_piece,
                                        data : current_piece_data.clone(),
                                    };
                                    match storage_sender.send(completed_piece).await {
                                        Ok(_k) => {
                                            // println!("Storage sent successfully : {:?}",k);
                                        },
                                        Err(e) => {
                                            println!("Storage sent Failed: {:?}",e);
                                            continue;
                                        },
                                    };

                                    current_piece = -1;
                                    current_offset = 0;
                                }
                                else {
                                    println!("Piece hash is not same wrong piece downloaded");
                                    break;
                                }
                            }
                        } 
                        else {
                            println!("Unhandled Message: Peer send msg {:?}",msg);
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
                    if current_piece == -1 {
                        // pieces_tried[current_piece as usize] = 1;
                        if ss.check_if_all_downloaded() {
                            println!("Download Completed!!!!!");
                        }
                        else{
                            println!("Peer {:?} have no pieces to download disconnecting from peer",peer.peer_id);
                        }
                        reader_task_handle.abort();
                        break;
                    }
                
                    let Ok(piece_index) = usize::try_from(current_piece) else {
                        pieces_tried[current_piece as usize] = 1;
                        current_piece = -1;
                        continue;
                    };
                    let is_available = peer_has_piece(&peer.peer_bitfield, piece_index);
                    if is_available {
                        ss.pieces[current_piece as usize].status = download_state::PieceStatus::Downloading;
                        println!("Downloading Piece {}",current_piece);
                        let piece_size = if current_piece == (nb_pieces as i64) - 1 {
                            (total_data_size - current_piece * piece_length) as usize
                        } else {
                            piece_length as usize
                        };
                        current_piece_data = vec![0u8; piece_size];
                        current_piece_size = piece_size;
                    }
                    else {
                        println!("Peer doesnot hava current piece: {}", is_available);
                        pieces_tried[current_piece as usize] = 1;
                        current_piece = -1;
                        continue;
                    }
                }

                if peer.interested == false {
                    let res = send_interested(&mut writer).await;
                    match res {
                        Ok(_) => {
                            println!("interested message sent successfully");
                            peer.interested = true;
                        },
                        Err(_) => {
                            println!("Error in sending interested message");
                            continue;
                        },
                    }
                }

                if !peer.is_choked{
                    if current_block_req_sent == false
                    {
                        let block_len = block_size_for_request(current_piece as u32,current_offset as u32,nb_pieces as u32,total_data_size as u64,piece_length as u32,BLOCK_SIZE as u32);
                      
                        match send_block_request(&mut writer,current_piece,current_offset as i64,block_len as i32).await {
                            Ok(_) => {
                                // println!("Sent block request for piece {} , offset {} , size {}",current_piece,current_offset,block_len);
                                current_block_req_sent = true;
                            },
                            Err(_) => {
                                println!("Error sending block request");
                            }
                            
                        }
                    }
                }
            }
        }
    }
    // let res = listen_peer(&mut peer,&info_hash,my_peerid_c,&mut stream).await;
}