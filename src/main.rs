use std::{fs};
use std::env;
use rand::distr::{Alphanumeric, SampleString};
use std::{net::SocketAddr};
use tokio::{sync::mpsc,net::TcpStream};
use std::sync::{Arc,Mutex};
use std::time::Duration;


mod bencoder;
mod torrentmeta;
mod peer;
mod comms;
mod utils;
mod download_state;
mod storage_manager;
mod benchmark;

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().skip(1).collect();
    let input = args.iter().find(|arg| arg.as_str() != "--benchmark").map(String::as_str);
    // Supplying a torrent path is treated as an explicit benchmark run; the
    // flag remains useful when benchmarking the bundled default torrent.
    let benchmark_enabled = input.is_some() || args.iter().any(|arg| arg == "--benchmark");
    let input = if let Some(input) = input {
        input
    } else {
        println!("Torrent File not provided using default file as sample");
        "test_torrent_files/ContinuousTimeBayesianNetworkReasoningandLearningEngine.torrent"
    };

    println!("Starting download for file {input}");
    if benchmark_enabled {
        println!("Benchmark reporting enabled");
    }

    let contents = match fs::read(input){
        Ok(data) => data,
        Err(err) => {
            println!("Unable to open file {input}: {err}");
            return ();
        }
    };


    // TORRENT FILE PARSIN
    let mut pos: usize = 0;
    let bvalue: bencoder::BValue = match bencoder::BValue::decode(&contents, &mut pos) {
        Ok(bvalue) => bvalue,
        Err(e) => {
            println!("Torrent file Bencoder parsing failed : {}",e);
            return ();
        }
    };

    let torrentmeta = match torrentmeta::TorrentMeta::create(&bvalue) {
        Ok(torrentmeta) => torrentmeta,
        // Err(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "Torrent metadata parsing failed")),
        Err(e) => {
            println!("Torrent metadata parsing failed : {}",e);
            return ();
        }
    };
    let info_hash = utils::get_sha1(&torrentmeta.info_hash);
    // torrentmeta.printer();
    

    // SHARED PIECES STATE CTEATION
    let nb_pieces: i64 = ((torrentmeta.info.length as f64) / (torrentmeta.info.piece_length as f64)).ceil() as i64;
    println!("nb_pieces : {}",nb_pieces);
    let shared_state = Arc::new(Mutex::new(download_state::DownloadState::init(nb_pieces)));


    // PEER COMMUNICATION
    let peer_id = Alphanumeric.sample_string(&mut rand::rng(), 20);
    let peer_info: peer::PeerInfo = match comms::comms_tracker::get_peer_info_from_tracker(&torrentmeta,peer_id.clone()).await {
        Ok(pi) => {pi},
        Err(e) => {
            println!("Error getting peerinfo from tracker: {:?}",e);
            return ();
        }
    };

    let benchmark = Arc::new(benchmark::Benchmark::new());
    let reporter = if benchmark_enabled {
        let metrics = benchmark.clone();
        Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            let mut previous_received_bytes = 0u64;
            loop {
                interval.tick().await;
                let (active, peak, received_bytes, verified_bytes) = metrics.snapshot();
                let speed = (received_bytes - previous_received_bytes) as f64 / (1024.0 * 1024.0);
                previous_received_bytes = received_bytes;
                println!("[benchmark] peers active={active} peak={peak} received={received_bytes} B verified={verified_bytes} B speed={speed:.2} MiB/s");
            }
        }))
    } else {
        None
    };

    // STORAGE HANDLERS
    let (storgee_tx, storage_rx ) = mpsc::channel::<storage_manager::CompletedPiece>(32);    
    let storage_manager = match storage_manager::StorageManager::init(torrentmeta.info.name, torrentmeta.info.piece_length as u64, shared_state.clone()).await {
        Ok(sm) => {sm},
        Err(e) => {
            println!("Error creating storage manager Err: {:?}",e);
            // return Err(io::Error::new(io::ErrorKind::Interrupted, "Error creating storage manager"));
            return ();
        },
    };
    tokio::spawn(storage_manager::storagemanager_listener_loop(storage_manager, storage_rx));
    
    
    // INDIVIDUAL PEER HANDLING
    let mut peer_join_handles = Vec::new();
    for i in 0..peer_info.peers.len(){

        let peer: peer::Peer = peer_info.get_peer_at_n(i);    
        let addr: SocketAddr = SocketAddr::new(peer.ip, peer.port);
    
        let stream = match TcpStream::connect(addr).await {
            Ok(str) => {str},
            Err(_) => {
                println!("Error connection to peer {:?}",&peer.peer_id);
                continue;
            }
        };

        let ss_c = Arc::clone(&shared_state);
        let info_hash_c = info_hash.clone();
        let my_peerid_c = peer_id.clone();
        let piece_length = torrentmeta.info.piece_length;
        let piece_hash = torrentmeta.info.pieces.clone();
        let total_data_size = torrentmeta.info.length;
        let peer_storgee_tx: mpsc::Sender<storage_manager::CompletedPiece>  = storgee_tx.clone();
        let benchmark_for_peer = benchmark.clone();
    
        let peer_handle = tokio::spawn(async move {
            comms::comms_peer::handle_peer(peer,info_hash_c, my_peerid_c,stream, ss_c,piece_length,piece_hash,peer_storgee_tx,total_data_size,benchmark_for_peer).await;
        });
        peer_join_handles.push(peer_handle);
    }

    for handle in peer_join_handles {
        handle.await.expect("Error in peer handle wait");
    }

    if let Some(reporter) = reporter {
        reporter.abort();
        let elapsed = benchmark.started.elapsed().as_secs_f64();
        let (active, peak, received_bytes, verified_bytes) = benchmark.snapshot();
        let average_speed = if elapsed > 0.0 { received_bytes as f64 / elapsed / (1024.0 * 1024.0) } else { 0.0 };
        println!("[benchmark] complete elapsed={elapsed:.2}s peers active={active} peak={peak} received={received_bytes} B verified={verified_bytes} B average={average_speed:.2} MiB/s");
    }
    
}
