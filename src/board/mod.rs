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
    pub cells: [Piece; BOARD_SIZE],
}

impl Board {
    pub fn new() -> Self {
        let mut cells = [Piece::OFF_BOARD; BOARD_SIZE];

        for l in 0..11 {
            for n in 0..11 {
                if Square::is_valid(l, n) {
                    cells[(l * 11 + n) as usize] = Piece::EMPTY;
                }
            }
        }

        Board { cells }
    }

    pub fn starting_pos() -> Self {
        let mut board = Self::new();

        let white_pawn = Piece::new(true, PieceType::Pawn);
        for i in 0..=4 {
            board.cells[i + 44] = white_pawn;
        }

        for i in 0..=4 {
            board.cells[i * 11 + 4] = white_pawn;
        }

        let black_pawn = Piece::new(false, PieceType::Pawn);
        for i in 6..=10 {
            board.cells[i + 77] = black_pawn;
        }

        for i in 6..=10 {
            board.cells[i * 11 + 6] = black_pawn;
        }

        for i in 0..=2 {
            board.cells[i * 12] = Piece::new(true, PieceType::Bishop);
            board.cells[120 - i * 12] = Piece::new(false, PieceType::Bishop);
        }

        board.cells[11] = Piece::new(true, PieceType::Queen);
        board.cells[119] = Piece::new(false, PieceType::Queen);

        board.cells[1] = Piece::new(true, PieceType::King);
        board.cells[109] = Piece::new(false, PieceType::King);

        let coords = (3, 0);
        board.cells[coords.0 * 11 + coords.1] = Piece::new(true, PieceType::Rook);
        board.cells[coords.1 * 11 + coords.0] = Piece::new(true, PieceType::Rook);

        let coords = (10, 2);
        board.cells[coords.0 * 11 + coords.1] = Piece::new(false, PieceType::Rook);
        board.cells[coords.1 * 11 + coords.0] = Piece::new(false, PieceType::Rook);

        let coords = (2, 0);
        board.cells[coords.0 * 11 + coords.1] = Piece::new(true, PieceType::Knight);
        board.cells[coords.1 * 11 + coords.0] = Piece::new(true, PieceType::Knight);

        let coords = (10, 3);
        board.cells[coords.0 * 11 + coords.1] = Piece::new(false, PieceType::Knight);
        board.cells[coords.1 * 11 + coords.0] = Piece::new(false, PieceType::Knight);

        board
    }

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        self.cells[(square.l * 11 + square.n) as usize] = piece;
    }
}
