use crate::board::{self, Board, PieceType, Square};

#[derive(Debug, Clone)]
pub struct Game {
    is_white: bool,
    en_passant_square: Option<Square>,
    board: Board,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move(u32);

impl Move {
    fn new(
        from: Square,
        to: Square,
        promoted_piece: PieceType,
        captured_piece: PieceType,
        was_pawn_double_push: bool,
        was_en_passant: bool,
        was_promotion: bool,
    ) -> Self {
        let mut final_val = u8::from(from) as u32;
        final_val <<= 7;
        final_val += u8::from(to) as u32;
        final_val <<= 3;
        final_val += promoted_piece as u8 as u32;
        final_val <<= 3;
        final_val += captured_piece as u8 as u32;
        final_val <<= 1;
        final_val += was_pawn_double_push as u32;
        final_val <<= 1;
        final_val += was_en_passant as u32;
        final_val <<= 1;
        final_val += was_promotion as u32;
        Self(final_val)
    }

    pub fn from(self) -> Square {
        unsafe { Square::try_from(((self.0 >> 16) & 0x7F) as u8).unwrap_unchecked() }
    }

    pub fn to(self) -> Square {
        unsafe { Square::try_from(((self.0 >> 9) & 0x7F) as u8).unwrap_unchecked() }
    }

    pub fn promoted_piece(self) -> PieceType {
        PieceType::from(((self.0 >> 6) & 0x7) as u8)
    }

    pub fn captured_piece(self) -> PieceType {
        PieceType::from(((self.0 >> 3) & 0x7) as u8)
    }

    pub fn was_pawn_double_push(self) -> bool {
        ((self.0 >> 2) & 0x1) != 0
    }

    pub fn was_en_passant(self) -> bool {
        ((self.0 >> 1) & 0x1) != 0
    }

    pub fn was_promotion(self) -> bool {
        (self.0 & 0x1) != 0
    }
}

impl Game {
    const ROOK_OFFSETS: &[i16] = &[1, -1, 11, -11, 12, -12];
    const BISHOP_OFFSETS: &[i16] = &[23, -23, 13, -13, 10, -10];
    const QUEEN_OFFSETS: &[i16] = &[1, -1, 11, -11, 12, -12, 23, -23, 13, -13, 10, -10];

    pub fn new() -> Self {
        Self {
            is_white: true,
            en_passant_square: None,
            board: Board::starting_pos(),
        }
    }

    pub fn from_pos(is_white: bool, board: Board) -> Self {
        Self {
            is_white,
            en_passant_square: None,
            board,
        }
    }

    fn get_sliding_moves(
        &self,
        start_idx: usize,
        moves: &mut [Move],
        write_idx: &mut usize,
    ) -> Result<(), ()> {
        let piece = self.board.get_piece(start_idx as u8);
        for offset in match piece.piece_type() {
            PieceType::Rook => Self::ROOK_OFFSETS,
            PieceType::Bishop => Self::BISHOP_OFFSETS,
            _ => Self::QUEEN_OFFSETS,
        } {
            let mut curr_idx = start_idx as i16;

            loop {
                curr_idx += offset;

                if curr_idx < 0 || curr_idx >= board::BOARD_SIZE as i16 {
                    break;
                }

                let target_piece = self.board.get_piece(curr_idx as u8);

                if target_piece.is_off_board() {
                    break;
                }
                if target_piece.is_empty() {
                    moves[*write_idx] = Move::new(
                        unsafe { Square::try_from(start_idx as u8).unwrap_unchecked() },
                        unsafe { Square::try_from(curr_idx as u8).unwrap_unchecked() },
                        PieceType::None,
                        PieceType::None,
                        false,
                        false,
                        false,
                    );
                    *write_idx += 1;
                } else {
                    if target_piece.is_white() != piece.is_white() {
                        moves[*write_idx] = Move::new(
                            unsafe { Square::try_from(start_idx as u8).unwrap_unchecked() },
                            unsafe { Square::try_from(curr_idx as u8).unwrap_unchecked() },
                            PieceType::None,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        );
                        *write_idx += 1;
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    fn get_knight_moves(&self, start_idx: usize, moves: &mut [Move], write_idx: &mut usize) {
        let piece = self.board.get_piece(start_idx as u8);
        for idx_offset in [34, -34, 35, -35, 25, -25, 14, -14, 9, -9, 21, -21] {
            let target_idx = start_idx as i16 + idx_offset;

            if target_idx < 0 || target_idx >= board::BOARD_SIZE as i16 {
                continue;
            }

            let target_piece = self.board.get_piece(target_idx as u8);

            if target_piece.is_off_board() { break; }
            if target_piece.is_empty() || target_piece.is_white() != piece.is_white() {
                moves[*write_idx] = Move::new(
                    unsafe { Square::try_from(start_idx as u8).unwrap_unchecked() },
                    unsafe { Square::try_from(target_idx as u8).unwrap_unchecked() },
                    PieceType::None,
                    target_piece.piece_type(),
                    false,
                    false,
                    false,
                );
                *write_idx += 1;
            }
        }
    }

    fn get_king_moves(&self, start_idx: usize, moves: &mut [Move], write_idx: &mut usize) {
        let piece = self.board.get_piece(start_idx as u8);
        for idx_offset in [
            1, 11, 10, 12, 23, 13, -1i16, -11i16, -10i16, -12i16, -23i16, -13i16,
        ] {
            let target_idx = start_idx as i16 + idx_offset;

            if target_idx < 0 || target_idx >= board::BOARD_SIZE as i16 {
                continue;
            }

            let target_piece = self.board.get_piece(target_idx as u8);

            if target_piece.is_off_board() {
                continue;
            }
            if target_piece.is_empty() || target_piece.is_white() != piece.is_white() {
                moves[*write_idx] = Move::new(
                    unsafe { Square::try_from(start_idx as u8).unwrap_unchecked() },
                    unsafe { Square::try_from(target_idx as u8).unwrap_unchecked() },
                    PieceType::None,
                    target_piece.piece_type(),
                    false,
                    false,
                    false,
                );
                *write_idx += 1;
            }
        }
    }

    pub fn get_pseudo_legal_moves(&self) -> Vec<Move> {
        todo!();
    }
}
