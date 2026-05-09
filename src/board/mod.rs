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

    pub fn new(is_white: bool, piece_type: PieceType) -> Self {
        let color_bit = if is_white { 1u8 << 3 } else { 0u8 };
        Self(color_bit | u8::from(piece_type))
    }

    pub fn is_white(self) -> bool {
        self.0 & COLOR_MASK == 0b1000
    }

    pub fn piece_type(self) -> PieceType {
        PieceType::from(self.0 & TYPE_MASK)
    }

    pub fn is_off_board(self) -> bool {
        self == Piece::OFF_BOARD
    }

    pub fn is_empty(self) -> bool {
        self == Piece::EMPTY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    l: u8,
    n: u8,
}

impl Square {
    pub fn new(l: i8, n: i8) -> Option<Self> {
        if Self::is_valid(l, n) {
            Some(Square {
                l: l as u8,
                n: n as u8,
            })
        } else {
            None
        }
    }

    pub unsafe fn new_unchecked(l: i8, n: i8) -> Self {
        Square {
            l: l as u8,
            n: n as u8,
        }
    }

    pub fn is_valid(l: i8, n: i8) -> bool {
        let diff = l - n;
        (0..11).contains(&l) && (0..11).contains(&n) && diff >= -5 && diff <= 5
    }
}

impl From<u8> for Square {
    fn from(value: u8) -> Self {
        Self {
            l: value / 11,
            n: value % 11,
        }
    }
}

impl From<Square> for u8 {
    fn from(value: Square) -> Self {
        value.l * 11 + value.n
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

    pub fn new() -> Self {
        Board { wp: 0, bp: 0, wr: 0, br: 0, wn: 0, bn: 0, wb: 0, bb: 0, wk: 0, bk: 0, wq: 0, bq: 0 }
    }

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

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        todo!();
    }
}
