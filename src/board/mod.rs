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

    #[inline]
    pub fn new(is_white: bool, piece_type: PieceType) -> Self {
        let color_bit = if is_white { 1u8 << 3 } else { 0u8 };
        Self(color_bit | u8::from(piece_type))
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    l: u8,
    n: u8,
}

impl Square {
    #[inline]
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

    #[inline]
    pub unsafe fn new_unchecked(l: i8, n: i8) -> Self {
        Square {
            l: l as u8,
            n: n as u8,
        }
    }

    #[inline]
    pub fn is_valid(l: i8, n: i8) -> bool {
        let diff = l - n;
        (0..11).contains(&l) && (0..11).contains(&n) && diff >= -5 && diff <= 5
    }

    #[inline]
    pub fn is_valid_idx(idx: u8) -> bool { Self::is_valid((idx / 11) as i8, (idx % 11) as i8) }
}

impl TryFrom<u8> for Square {
    type Error = ();
    #[inline]
    fn try_from(value: u8) -> Result<Self, ()> {
        let l = value / 11;
        let n = value % 11;
        if Self::is_valid(l as i8, n as i8) { Ok(Self { l, n }) }
        else { Err(()) }
    }
}

impl From<Square> for u8 {
    #[inline]
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

    #[inline]
    pub fn clear_square(&mut self, idx: u8) {
        let mask = !(1u128 << idx);
        self.wp &= mask;
        self.bp &= mask;
        self.wr &= mask;
        self.br &= mask;
        self.wn &= mask;
        self.bn &= mask;
        self.wb &= mask;
        self.bb &= mask;
        self.wq &= mask;
        self.bq &= mask;
        self.wk &= mask;
        self.bk &= mask;
    }

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        let idx = u8::from(square);
        let select_mask: u128 = !(1 << idx);
        self.clear_square(idx);

        match (piece.piece_type(), piece.is_white()) {
            (PieceType::Pawn, true) => self.wp |= !select_mask,
            (PieceType::Pawn, false) => self.bp |= !select_mask,
            (PieceType::Rook, true) => self.wr |= !select_mask,
            (PieceType::Rook, false) => self.br |= !select_mask,
            (PieceType::Knight, true) => self.wn |= !select_mask,
            (PieceType::Knight, false) => self.bn |= !select_mask,
            (PieceType::Bishop, true) => self.wb |= !select_mask,
            (PieceType::Bishop, false) => self.bb |= !select_mask,
            (PieceType::Queen, true) => self.wq |= !select_mask,
            (PieceType::Queen, false) => self.bq |= !select_mask,
            (PieceType::King, true) => self.wk |= !select_mask,
            (PieceType::King, false) => self.bk |= !select_mask,
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
        let existence = [self.wk, self.bk, self.wp, self.bp, self.wr, self.br, self.wn, self.bn, self.wb, self.bb, self.wq, self.bq].into_iter()
            .map(|bb| (bb & (1 << idx)) >> idx == 1);

        let mut piece_type = 0;
        let mut is_white = true;

        if !Square::is_valid_idx(idx) { return Piece::OFF_BOARD; }

        for val in existence {
            if val {
                return Piece::new(is_white, PieceType::from(piece_type));
            }
            
            if !is_white {
                piece_type += 1;
            }

            is_white = !is_white;
        }

        return Piece::EMPTY;
    }
}
