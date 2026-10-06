

#[derive(PartialEq, Eq)]
pub enum PieceStatus {
    Pending,
    Downloading,
    Completed,
}

pub struct Piece{
    pub id: i64,
    pub status: PieceStatus,
}

pub struct DownloadState{
    pub piece_count: u64,
    pub pieces: Vec<Piece>,
}

impl DownloadState {
    pub fn init(nb_pieces: i64) -> Self {
        let mut vec_pieces:  Vec<Piece> = Vec::new();
        for num in 0..nb_pieces {
            vec_pieces.push(Piece{
                id: num,
                status: PieceStatus::Pending,
            });
        }
        let ds = DownloadState{
            pieces:vec_pieces,
            piece_count: nb_pieces as u64,
        };
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
}

