# RustBitTorrent

A minimal BitTorrent client written in Rust. It reads a `.torrent` file, requests peers from its tracker, and downloads pieces over TCP.

## Features

- Bencode decoding for torrent metadata and tracker responses
- Tracker announce and peer discovery
- Peer communication using the BitTorrent peer-wire protocol:
  - Choke / Unchoke
  - Interested / Not Interested
  - Bitfield
  - Request
  - Piece
- Piece selection, block requests, and SHA-1 piece verification
- Concurrent peer handling, piece scheduling and shared download state
- Asynchronous file storage

## How it works

1. Reads a torrent file path from the command line (defaults to the bundled sample torrent).
2. Decodes the metadata and calculates the info hash and piece count.
3. Announces to the tracker and parses the returned peer list.
4. Connects to peers, completes the handshake, and requests blocks for available pieces.
5. Verifies each complete piece against the torrent's SHA-1 hashes, then writes verified data at its file offset.

## Run

```sh
cargo run -- path/to/file.torrent
```

Without a path, the client uses `test_torrent_files/ContinuousTimeBayesianNetworkReasoningandLearningEngine.torrent`.

## Current Limitations

1. Seeding is not implemented
2. Resume support is not implemented
3. Multi-file torrents are not currently supported
