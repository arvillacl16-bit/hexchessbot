// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
