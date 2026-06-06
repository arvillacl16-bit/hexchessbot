#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece(u8);

pub const BOARD_SIZE: usize = 121;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceType {
    None,
    King,
    Pawn,
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl From<u8> for PieceType {
    fn from(value: u8) -> Self {
        match value {
            1 => PieceType::King,
            2 => PieceType::Pawn,
            3 => PieceType::Queen,
            4 => PieceType::Rook,
            5 => PieceType::Bishop,
            6 => PieceType::Knight,
            _ => PieceType::None,
        }
    }
}

impl From<PieceType> for u8 {
    fn from(piece: PieceType) -> Self {
        piece as u8
    }
}

const COLOR_MASK: u8 = 0b0000_1000;
const TYPE_MASK: u8 = 0b0000_0111;

impl Piece {
    pub const OFF_BOARD: Piece = Piece(128);
    pub const EMPTY: Piece = Piece(0);

    pub const WHITE_PAWN: Piece = Piece(0b1010);
    pub const BLACK_PAWN: Piece = Piece(0b0010);
    pub const WHITE_KING: Piece = Piece(0b1001);
    pub const BLACK_KING: Piece = Piece(0b0001);
    pub const WHITE_KNIGHT: Piece = Piece(0b1110);
    pub const BLACK_KNIGHT: Piece = Piece(0b0110);
    pub const WHITE_BISHOP: Piece = Piece(0b1101);
    pub const BLACK_BISHOP: Piece = Piece(0b0101);
    pub const WHITE_ROOK: Piece = Piece(0b1100);
    pub const BLACK_ROOK: Piece = Piece(0b0100);
    pub const WHITE_QUEEN: Piece = Piece(0b1011);
    pub const BLACK_QUEEN: Piece = Piece(0b0011);

    #[inline]
    pub fn is_white(self) -> bool {
        self.0 & COLOR_MASK == 0b1000
    }

    #[inline]
    pub fn piece_type(self) -> PieceType {
        PieceType::from(self.0 & TYPE_MASK)
    }

    #[inline]
    pub fn is_off_board(self) -> bool {
        self == Piece::OFF_BOARD
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self == Piece::EMPTY
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    wp: u128,
    bp: u128,
    wr: u128,
    br: u128,
    wn: u128,
    bn: u128,
    wb: u128,
    bb: u128,
    wk: u128,
    bk: u128,
    wq: u128,
    bq: u128,
}

impl Board {
    const ON_BOARD: u128 = 0b00000001111110000011111110000111111110001111111110011111111110111111111110111111111100111111111000111111110000111111100000111111;
    #[inline]
    pub fn is_invalid_idx(idx: u8) -> bool {
        matches!(idx, 6 | 7 | 8 | 9 | 10 | 18 | 19 | 20 | 21 | 30 | 31 | 32 | 42 | 43 | 54 | 66 | 77 | 78 | 88 | 89 | 90 | 99 | 100 | 101 | 102 | 110 | 111 | 112 | 113 | 114)
    }

    #[inline]
    pub fn is_valid_idx(idx: u8) -> bool {
        !Self::is_invalid_idx(idx)
    }
    
    #[inline]
    pub fn new() -> Self {
        Board {
            wp: 0,
            bp: 0,
            wr: 0,
            br: 0,
            wn: 0,
            bn: 0,
            wb: 0,
            bb: 0,
            wk: 0,
            bk: 0,
            wq: 0,
            bq: 0,
        }
    }

    #[inline]
    pub fn starting_pos() -> Self {
        Board {
            wp: 0b00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00111110000_00010000000_00010000000_00010000000_00001000000,
            bp: 0b00000100000_00000010000_00000010000_00000010000_00001111100_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000,
            wr: 0b00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_10000000000_00000000000_00000000000_00010000000,
            br: 0b00000001000_00000000000_00000000000_00000000001_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000,
            wn: 0b00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_10000000000_00000000000_00100000000,
            bn: 0b00000000100_00000000000_00000000100_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000,
            wb: 0b00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000010_00000000001_00000000001,
            bb: 0b10000000000_10000000000_01000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000,
            wq: 0b00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_10000000000_00000000000,
            bq: 0b00000000000_01000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000,
            wk: 0b00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000010,
            bk: 0b01000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000,
        }
    }

    pub fn set_piece(&mut self, idx: u8, piece: Piece) {
        let select_mask: u128 = 1 << idx;

        match (piece.piece_type(), piece.is_white()) {
            (PieceType::Pawn, true) => self.wp |= select_mask,
            (PieceType::Pawn, false) => self.bp |= select_mask,
            (PieceType::Rook, true) => self.wr |= select_mask,
            (PieceType::Rook, false) => self.br |= select_mask,
            (PieceType::Knight, true) => self.wn |= select_mask,
            (PieceType::Knight, false) => self.bn |= select_mask,
            (PieceType::Bishop, true) => self.wb |= select_mask,
            (PieceType::Bishop, false) => self.bb |= select_mask,
            (PieceType::Queen, true) => self.wq |= select_mask,
            (PieceType::Queen, false) => self.bq |= select_mask,
            (PieceType::King, true) => self.wk |= select_mask,
            (PieceType::King, false) => self.bk |= select_mask,
            _ => {}
        }
    }

    #[inline]
    pub fn white_pieces(&self) -> u128 {
        self.wk | self.wp | self.wr | self.wn | self.wb | self.wq
    }

    #[inline]
    pub fn black_pieces(&self) -> u128 {
        self.bk | self.bp | self.br | self.bn | self.bb | self.bq
    }

    #[inline]
    pub fn white_pawns(&self) -> u128 {
        self.wp
    }
    #[inline]
    pub fn black_pawns(&self) -> u128 {
        self.bp
    }
    #[inline]
    pub fn white_kings(&self) -> u128 {
        self.wk
    }
    #[inline]
    pub fn black_kings(&self) -> u128 {
        self.bk
    }
    #[inline]
    pub fn white_rooks(&self) -> u128 {
        self.wr
    }
    #[inline]
    pub fn black_rooks(&self) -> u128 {
        self.br
    }
    #[inline]
    pub fn white_knights(&self) -> u128 {
        self.wn
    }
    #[inline]
    pub fn black_knights(&self) -> u128 {
        self.bn
    }
    #[inline]
    pub fn white_bishops(&self) -> u128 {
        self.wb
    }
    #[inline]
    pub fn black_bishops(&self) -> u128 {
        self.bb
    }
    #[inline]
    pub fn white_queens(&self) -> u128 {
        self.wq
    }
    #[inline]
    pub fn black_queens(&self) -> u128 {
        self.bq
    }

    pub fn get_piece(&self, idx: u8) -> Piece {
        if !Self::is_valid_idx(idx) {
            return Piece::OFF_BOARD;
        }

        let mask = 1u128 << idx;
        let white = self.white_pieces();
        let black = self.black_pieces();

        if ((white | black) & mask) == 0 {
            return Piece::EMPTY;
        }

        if (white & mask) != 0 {
            if (self.wp & mask) != 0 { return Piece::WHITE_PAWN; }
            if (self.wn & mask) != 0 { return Piece::WHITE_KNIGHT; }
            if (self.wb & mask) != 0 { return Piece::WHITE_BISHOP; }
            if (self.wr & mask) != 0 { return Piece::WHITE_ROOK; }
            if (self.wq & mask) != 0 { return Piece::WHITE_QUEEN; }
            if (self.wk & mask) != 0 { return Piece::WHITE_KING; }
        } else {
            if (self.bp & mask) != 0 { return Piece::BLACK_PAWN; }
            if (self.bn & mask) != 0 { return Piece::BLACK_KNIGHT; }
            if (self.bb & mask) != 0 { return Piece::BLACK_BISHOP; }
            if (self.br & mask) != 0 { return Piece::BLACK_ROOK; }
            if (self.bq & mask) != 0 { return Piece::BLACK_QUEEN; }
            if (self.bk & mask) != 0 { return Piece::BLACK_KING; }
        }

        Piece::EMPTY
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
