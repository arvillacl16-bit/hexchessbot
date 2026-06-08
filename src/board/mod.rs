// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
    pub(crate) wp: u128,
    pub(crate) bp: u128,
    pub(crate) wr: u128,
    pub(crate) br: u128,
    pub(crate) wn: u128,
    pub(crate) bn: u128,
    pub(crate) wb: u128,
    pub(crate) bb: u128,
    pub(crate) wk: u128,
    pub(crate) bk: u128,
    pub(crate) wq: u128,
    pub(crate) bq: u128,
    white: u128,
    black: u128,
}

impl Board {
    const ON_BOARD: u128 = 0b00000001111110000011111110000111111110001111111110011111111110111111111110111111111100111111111000111111110000111111100000111111;
    #[inline]
    pub fn is_valid_idx(idx: u8) -> bool {
        if idx > 120 { return false; }
        (1u128 << idx) & Self::ON_BOARD != 0
    }

    #[inline]
    pub fn is_invalid_idx(idx: u8) -> bool {
        !Self::is_valid_idx(idx)
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
            white: 0,
            black: 0,
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
            white: 0b00000000000_00000000000_00000000000_00000000000_00000000000_00000000000_00111110000_10010000000_10010000010_11010000001_00111000011,
            black: 0b11000101000_11000010000_01000010100_00000010001_00001111100_00000000000_00000000000_00000000000_00000000000_00000000000_00000000000,
        }
    }

    fn sync_total_bb(&mut self) {
        self.white = self.wp | self.wn | self.wb | self.wr | self.wq | self.wk;
        self.black = self.bp | self.bn | self.bb | self.br | self.bq | self.bk;
    }

    pub fn set_piece(&mut self, idx: u8, piece: Piece) {
        let select_mask = 1u128 << idx;
        match self.get_piece(idx) {
            Piece::WHITE_PAWN => self.wp &= !select_mask,
            Piece::BLACK_PAWN => self.bp &= !select_mask,
            Piece::WHITE_ROOK => self.wr &= !select_mask,
            Piece::BLACK_ROOK => self.br &= !select_mask,
            Piece::WHITE_KNIGHT => self.wn &= !select_mask,
            Piece::BLACK_KNIGHT => self.bn &= !select_mask,
            Piece::WHITE_BISHOP => self.wb &= !select_mask,
            Piece::BLACK_BISHOP => self.bb &= !select_mask,
            Piece::WHITE_QUEEN => self.wq &= !select_mask,
            Piece::BLACK_QUEEN => self.bq &= !select_mask,
            Piece::WHITE_KING => self.wk &= !select_mask,
            Piece::BLACK_KING => self.bk &= !select_mask,
            _ => {}
        }

        match piece {
            Piece::WHITE_PAWN => self.wp |= select_mask,
            Piece::BLACK_PAWN => self.bp |= select_mask,
            Piece::WHITE_ROOK => self.wr |= select_mask,
            Piece::BLACK_ROOK => self.br |= select_mask,
            Piece::WHITE_KNIGHT => self.wn |= select_mask,
            Piece::BLACK_KNIGHT => self.bn |= select_mask,
            Piece::WHITE_BISHOP => self.wb |= select_mask,
            Piece::BLACK_BISHOP => self.bb |= select_mask,
            Piece::WHITE_QUEEN => self.wq |= select_mask,
            Piece::BLACK_QUEEN => self.bq |= select_mask,
            Piece::WHITE_KING => self.wk |= select_mask,
            Piece::BLACK_KING => self.bk |= select_mask,
            _ => {}
        }

        self.sync_total_bb();
    }

    #[inline]
    pub fn white_pieces(&self) -> u128 {
        self.white
    }

    #[inline]
    pub fn black_pieces(&self) -> u128 {
        self.black
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
        let occupancy = self.white | self.black;
        if (occupancy & mask) != 0 {
            return Piece::EMPTY;
        }

        let is_white = ((self.white & mask) != 0) as u8;

        let is_king = ((self.wk | self.bk) & mask != 0) as u8;
        let is_pawn = ((self.wp | self.bp) & mask != 0) as u8;
        let is_queen = ((self.wq | self.bq) & mask != 0) as u8;
        let is_rook = ((self.wr | self.br) & mask != 0) as u8;
        let is_bishop = ((self.wb | self.bb) & mask != 0) as u8;
        let is_knight = ((self.wn | self.bn) & mask != 0) as u8;

        let piece_type = (is_king * 1)
            | (is_pawn * 2)
            | (is_queen * 3)
            | (is_rook * 4)
            | (is_bishop * 5)
            | (is_knight * 6);

        Piece((is_white << 3 | piece_type))
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
