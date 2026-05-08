pub mod board;
pub mod moves;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece() {
        let piece = board::Piece::new(true, board::PieceType::Pawn);
        assert_eq!(piece.piece_type(), board::PieceType::Pawn);
    }
}
