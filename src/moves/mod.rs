// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::board::{self, Board, PieceType};

#[derive(Debug, Clone)]
pub struct Game {
    is_white: bool,
    en_passant_square: Option<u8>,
    board: Board,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Move(u32);

#[derive(Debug, Clone, Copy)]
struct Magic {
    mask: u128,
    magic: u128,
    shift: u32,
    offset: usize,
}

impl Move {
    fn new(
        from: u8,
        to: u8,
        promoted_piece: PieceType,
        captured_piece: PieceType,
        was_pawn_double_push: bool,
        was_en_passant: bool,
        was_promotion: bool,
    ) -> Self {
        let final_val = ((from as u32) << 16)
            | ((to as u32) << 9)
            | ((promoted_piece as u8 as u32) << 6)
            | ((captured_piece as u8 as u32) << 3)
            | ((was_pawn_double_push as u32) << 2)
            | ((was_en_passant as u32) << 1)
            | (was_promotion as u32);

        Self(final_val)
    }

    pub fn from(self) -> u8 {
        ((self.0 >> 16) & 0x7F) as u8
    }

    pub fn to(self) -> u8 {
        ((self.0 >> 9) & 0x7F) as u8
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

static mut BISHOP_ATTACKS_DB: [u128; 20480] = [0; 20480];
static mut ROOK_R_ATTACKS_DB: [u128; 24576] = [0; 24576];
static mut ROOK_C_ATTACKS_DB: [u128; 24576] = [0; 24576];
static mut ROOK_X_ATTACKS_DB: [u128; 24576] = [0; 24576];

impl Game {
    const KNIGHT_ATTACKS: [u128; 121] = [
        /* A1 idx  0 */ 0x00000000000000000000000C02004000, // 4 moves
        /* A2 idx  1 */ 0x00000000000000000000001804408000, // 5 moves
        /* A3 idx  2 */ 0x00000000000000000000003008810800, // 6 moves
        /* A4 idx  3 */ 0x00000000000000000000006011021000, // 6 moves
        /* A5 idx  4 */ 0x0000000000000000000000C022002000, // 5 moves
        /* A6 idx  5 */ 0x00000000000000000000018004004000, // 4 moves
        /* A7 off */ 0,
        /* A8 off */ 0,
        /* A9 off */ 0,
        /* A10 off */ 0,
        /* A11 off */ 0,
        /* B1 idx 11 */ 0x00000000000000000000601002000004, // 5 moves
        /* B2 idx 12 */ 0x00000000000000000000C02204000008, // 6 moves
        /* B3 idx 13 */ 0x00000000000000000001804408400010, // 7 moves
        /* B4 idx 14 */ 0x00000000000000000003008810800021, // 8 moves
        /* B5 idx 15 */ 0x00000000000000000006011021000002, // 7 moves
        /* B6 idx 16 */ 0x0000000000000000000C022002000004, // 6 moves
        /* B7 idx 17 */ 0x00000000000000000018004004000008, // 5 moves
        /* B8 off */ 0,
        /* B9 off */ 0,
        /* B10 off */ 0,
        /* B11 off */ 0,
        /* C1 idx 22 */ 0x00000000000000000300801000002002, // 6 moves
        /* C2 idx 23 */ 0x00000000000000000601102000004004, // 7 moves
        /* C3 idx 24 */ 0x00000000000000000C02204200008008, // 8 moves
        /* C4 idx 25 */ 0x00000000000000001804408400010811, // 10 moves
        /* C5 idx 26 */ 0x00000000000000003008810800021022, // 10 moves
        /* C6 idx 27 */ 0x00000000000000006011021000002004, // 8 moves
        /* C7 idx 28 */ 0x0000000000000000C022002000004008, // 7 moves
        /* C8 idx 29 */ 0x00000000000000018004004000008010, // 6 moves
        /* C9 off */ 0,
        /* C10 off */ 0,
        /* C11 off */ 0,
        /* D1 idx 33 */ 0x00000000000000180400800001001000, // 6 moves
        /* D2 idx 34 */ 0x00000000000000300881000002002001, // 8 moves
        /* D3 idx 35 */ 0x00000000000000601102100004004003, // 10 moves
        /* D4 idx 36 */ 0x00000000000000C02204200008408806, // 12 moves
        /* D5 idx 37 */ 0x0000000000000180440840001081100C, // 12 moves
        /* D6 idx 38 */ 0x00000000000003008810800021022018, // 12 moves
        /* D7 idx 39 */ 0x00000000000006011021000002004030, // 10 moves
        /* D8 idx 40 */ 0x0000000000000C022002000004008020, // 8 moves
        /* D9 idx 41 */ 0x00000000000018004004000008010000, // 6 moves
        /* D10 off */ 0,
        /* D11 off */ 0,
        /* E1 idx 44 */ 0x00000000000080200400000800800000, // 5 moves
        /* E2 idx 45 */ 0x00000000000180400800001001000800, // 7 moves
        /* E3 idx 46 */ 0x00000000000300881080002002001800, // 10 moves
        /* E4 idx 47 */ 0x00000000000601102100004204403000, // 12 moves
        /* E5 idx 48 */ 0x00000000000C02204200008408806000, // 12 moves
        /* E6 idx 49 */ 0x0000000000180440840001081100C000, // 12 moves
        /* E7 idx 50 */ 0x00000000003008810800021022018000, // 12 moves
        /* E8 idx 51 */ 0x00000000006011021000002004030000, // 10 moves
        /* E9 idx 52 */ 0x0000000000C002002000004008020000, // 7 moves
        /* E10 idx 53 */ 0x00000000008004004000008010000000, // 5 moves
        /* E11 off */ 0,
        /* F1 idx 55 */ 0x00000000000100200000400400000000, // 4 moves
        /* F2 idx 56 */ 0x00000000080200400000800800400000, // 6 moves
        /* F3 idx 57 */ 0x00000000180400800001001000C00000, // 8 moves
        /* F4 idx 58 */ 0x00000000300881080002102201800000, // 12 moves
        /* F5 idx 59 */ 0x00000000601102100004204403000000, // 12 moves
        /* F6 idx 60 */ 0x00000000C02204200008408806000000, // 12 moves
        /* F7 idx 61 */ 0x0000000180440840001081100C000000, // 12 moves
        /* F8 idx 62 */ 0x00000003008810800021022018000000, // 12 moves
        /* F9 idx 63 */ 0x00000006001001000002004030000000, // 8 moves
        /* F10 idx 64 */ 0x00000004002002000004008020000000, // 6 moves
        /* F11 idx 65 */ 0x00000000004004000008010000000000, // 4 moves
        /* G1 off */ 0,
        /* G2 idx 67 */ 0x00000000100200000400400200000000, // 5 moves
        /* G3 idx 68 */ 0x0000008020040000080080060000000, // 7 moves
        /* G4 idx 69 */ 0x00000180400800001081100C00000000, // 10 moves
        /* G5 idx 70 */ 0x00000300881080002102201800000000, // 12 moves
        /* G6 idx 71 */ 0x00000601102100004204403000000000, // 12 moves
        /* G7 idx 72 */ 0x00000C02204200008408806000000000, // 12 moves
        /* G8 idx 73 */ 0x0000180440840001081100C000000000, // 12 moves
        /* G9 idx 74 */ 0x00003000800800021022018000000000, // 10 moves
        /* G10 idx 75 */ 0x00002001001000002004030000000000, // 7 moves
        /* G11 idx 76 */ 0x00000002002000004008020000000000, // 5 moves
        /* H1 off */ 0,
        /* H2 off */ 0,
        /* H3 idx 79 */ 0x00000100200000400400300000000000, // 6 moves
        /* H4 idx 80 */ 0x00080200400000800880600000000000, // 8 moves
        /* H5 idx 81 */ 0x00180400800001081100C00000000000, // 10 moves
        /* H6 idx 82 */ 0x00300881080002102201800000000000, // 12 moves
        /* H7 idx 83 */ 0x00601102100004204403000000000000, // 12 moves
        /* H8 idx 84 */ 0x00C02204200008408806000000000000, // 12 moves
        /* H9 idx 85 */ 0x0180040040001081100C000000000000, // 10 moves
        /* H10 idx 86 */ 0x01000800800001022018000000000000, // 8 moves
        /* H11 idx 87 */ 0x00001001000002004030000000000000, // 6 moves
        /* I1 off */ 0,
        /* I2 off */ 0,
        /* I3 off */ 0,
        /* I4 idx 91 */ 0x00100200000400400300000000000000, // 6 moves
        /* I5 idx 92 */ 0x00200400000800880600000000000000, // 7 moves
        /* I6 idx 93 */ 0x00400800001081100C00000000000000, // 8 moves
        /* I7 idx 94 */ 0x00881080002102201800000000000000, // 10 moves
        /* I8 idx 95 */ 0x01102100004204403000000000000000, // 10 moves
        /* I9 idx 96 */ 0x00200200008408806000000000000000, // 8 moves
        /* I10 idx 97 */ 0x0040040000081100C000000000000000, // 7 moves
        /* I11 idx 98 */ 0x00800800001002018000000000000000, // 6 moves
        /* J1 off */ 0,
        /* J2 off */ 0,
        /* J3 off */ 0,
        /* J4 off */ 0,
        /* J5 idx103 */ 0x00200000400400300000000000000000, // 5 moves
        /* J6 idx104 */ 0x00400000800880600000000000000000, // 6 moves
        /* J7 idx105 */ 0x00800001081100C00000000000000000, // 7 moves
        /* J8 idx106 */ 0x01080002102201800000000000000000, // 8 moves
        /* J9 idx107 */ 0x00100004204403000000000000000000, // 7 moves
        /* J10 idx108 */ 0x00200000408806000000000000000000, // 6 moves
        /* J11 idx109 */ 0x0040000080100C000000000000000000, // 5 moves
        /* K1 off */ 0,
        /* K2 off */ 0,
        /* K3 off */ 0,
        /* K4 off */ 0,
        /* K5 off */ 0,
        /* K6 idx115 */ 0x00000400400300000000000000000000, // 4 moves
        /* K7 idx116 */ 0x00000800880600000000000000000000, // 5 moves
        /* K8 idx117 */ 0x00001081100C00000000000000000000, // 6 moves
        /* K9 idx118 */ 0x00002102201800000000000000000000, // 6 moves
        /* K10 idx119 */ 0x00000204403000000000000000000000, // 5 moves
        /* K11 idx120 */ 0x00000400806000000000000000000000, // 4 moves
    ];
    const KING_ATTACKS: [u128; 121] = [
        /* A1 idx  0 */ 0x00000000000000000000000000803802, // 5 moves
        /* A2 idx  1 */ 0x00000000000000000000000001007805, // 7 moves
        /* A3 idx  2 */ 0x0000000000000000000000000200F00A, // 7 moves
        /* A4 idx  3 */ 0x0000000000000000000000000401E014, // 7 moves
        /* A5 idx  4 */ 0x0000000000000000000000000803C028, // 7 moves
        /* A6 idx  5 */ 0x00000000000000000000000010038010, // 5 moves
        /* A7 off */ 0,
        /* A8 off */ 0,
        /* A9 off */ 0,
        /* A10 off */ 0,
        /* A11 off */ 0,
        /* B1 idx 11 */ 0x00000000000000000000000401C01003, // 7 moves
        /* B2 idx 12 */ 0x00000000000000000000000803C02807, // 10 moves
        /* B3 idx 13 */ 0x0000000000000000000000100780500F, // 11 moves
        /* B4 idx 14 */ 0x0000000000000000000000200F00A01E, // 11 moves
        /* B5 idx 15 */ 0x0000000000000000000000401E01403C, // 11 moves
        /* B6 idx 16 */ 0x0000000000000000000000803C028038, // 10 moves
        /* B7 idx 17 */ 0x00000000000000000000010038010030, // 7 moves
        /* B8 off */ 0,
        /* B9 off */ 0,
        /* B10 off */ 0,
        /* B11 off */ 0,
        /* C1 idx 22 */ 0x00000000000000000000200E00801800, // 7 moves
        /* C2 idx 23 */ 0x00000000000000000000401E01403801, // 11 moves
        /* C3 idx 24 */ 0x00000000000000000000803C02807802, // 12 moves
        /* C4 idx 25 */ 0x0000000000000000000100780500F004, // 12 moves
        /* C5 idx 26 */ 0x0000000000000000000200F00A01E008, // 12 moves
        /* C6 idx 27 */ 0x0000000000000000000401E01403C010, // 12 moves
        /* C7 idx 28 */ 0x0000000000000000000803C028038020, // 11 moves
        /* C8 idx 29 */ 0x00000000000000000010038010030000, // 7 moves
        /* C9 off */ 0,
        /* C10 off */ 0,
        /* C11 off */ 0,
        /* D1 idx 33 */ 0x00000000000000000100700400C00000, // 7 moves
        /* D2 idx 34 */ 0x00000000000000000200F00A01C00800, // 11 moves
        /* D3 idx 35 */ 0x00000000000000000401E01403C01000, // 12 moves
        /* D4 idx 36 */ 0x00000000000000000803C02807802000, // 12 moves
        /* D5 idx 37 */ 0x0000000000000000100780500F004000, // 12 moves
        /* D6 idx 38 */ 0x0000000000000000200F00A01E008000, // 12 moves
        /* D7 idx 39 */ 0x0000000000000000401E01403C010000, // 12 moves
        /* D8 idx 40 */ 0x0000000000000000803C028038020000, // 11 moves
        /* D9 idx 41 */ 0x00000000000000010038010030000000, // 7 moves
        /* D10 off */ 0,
        /* D11 off */ 0,
        /* E1 idx 44 */ 0x00000000000000080380200600000000, // 7 moves
        /* E2 idx 45 */ 0x00000000000000100780500E00400000, // 11 moves
        /* E3 idx 46 */ 0x00000000000000200F00A01E00800000, // 12 moves
        /* E4 idx 47 */ 0x00000000000000401E01403C01000000, // 12 moves
        /* E5 idx 48 */ 0x00000000000000803C02807802000000, // 12 moves
        /* E6 idx 49 */ 0x0000000000000100780500F004000000, // 12 moves
        /* E7 idx 50 */ 0x0000000000000200F00A01E008000000, // 12 moves
        /* E8 idx 51 */ 0x0000000000000401E01403C010000000, // 12 moves
        /* E9 idx 52 */ 0x0000000000000803C028038020000000, // 11 moves
        /* E10 idx 53 */ 0x00000000000010038010030000000000, // 7 moves
        /* E11 off */ 0,
        /* F1 idx 55 */ 0x00000000000000180100300000000000, // 5 moves
        /* F2 idx 56 */ 0x00000000000080380280700200000000, // 10 moves
        /* F3 idx 57 */ 0x00000000000100780500F00400000000, // 12 moves
        /* F4 idx 58 */ 0x00000000000200F00A01E00800000000, // 12 moves
        /* F5 idx 59 */ 0x00000000000401E01403C01000000000, // 12 moves
        /* F6 idx 60 */ 0x00000000000803C02807802000000000, // 12 moves
        /* F7 idx 61 */ 0x0000000000100780500F004000000000, // 12 moves
        /* F8 idx 62 */ 0x0000000000200F00A01E008000000000, // 12 moves
        /* F9 idx 63 */ 0x0000000000401E01403C010000000000, // 12 moves
        /* F10 idx 64 */ 0x0000000000801C028038020000000000, // 10 moves
        /* F11 idx 65 */ 0x00000000000018010030000000000000, // 5 moves
        /* G1 off */ 0,
        /* G2 idx 67 */ 0x00000000000180100380100000000000, // 7 moves
        /* G3 idx 68 */ 0x00000000080380280780200000000000, // 11 moves
        /* G4 idx 69 */ 0x00000000100780500F00400000000000, // 12 moves
        /* G5 idx 70 */ 0x00000000200F00A01E00800000000000, // 12 moves
        /* G6 idx 71 */ 0x00000000401E01403C01000000000000, // 12 moves
        /* G7 idx 72 */ 0x00000000803C02807802000000000000, // 12 moves
        /* G8 idx 73 */ 0x0000000100780500F004000000000000, // 12 moves
        /* G9 idx 74 */ 0x0000000200F00A01E008000000000000, // 12 moves
        /* G10 idx 75 */ 0x0000000400E01403C010000000000000, // 11 moves
        /* G11 idx 76 */ 0x0000000000C008038020000000000000, // 7 moves
        /* H1 off */ 0,
        /* H2 off */ 0,
        /* H3 idx 79 */ 0x00000000180100380100000000000000, // 7 moves
        /* H4 idx 80 */ 0x00000080380280780200000000000000, // 11 moves
        /* H5 idx 81 */ 0x00000100780500F00400000000000000, // 12 moves
        /* H6 idx 82 */ 0x00000200F00A01E00800000000000000, // 12 moves
        /* H7 idx 83 */ 0x00000401E01403C01000000000000000, // 12 moves
        /* H8 idx 84 */ 0x00000803C02807802000000000000000, // 12 moves
        /* H9 idx 85 */ 0x0000100780500F004000000000000000, // 12 moves
        /* H10 idx 86 */ 0x0000200700A01E008000000000000000, // 11 moves
        /* H11 idx 87 */ 0x0000000600401C010000000000000000, // 7 moves
        /* I1 off */ 0,
        /* I2 off */ 0,
        /* I3 off */ 0,
        /* I4 idx 91 */ 0x00000180100380100000000000000000, // 7 moves
        /* I5 idx 92 */ 0x00080380280780200000000000000000, // 11 moves
        /* I6 idx 93 */ 0x00100780500F00400000000000000000, // 12 moves
        /* I7 idx 94 */ 0x00200F00A01E00800000000000000000, // 12 moves
        /* I8 idx 95 */ 0x00401E01403C01000000000000000000, // 12 moves
        /* I9 idx 96 */ 0x00803C02807802000000000000000000, // 12 moves
        /* I10 idx 97 */ 0x0100380500F004000000000000000000, // 11 moves
        /* I11 idx 98 */ 0x0000300200E008000000000000000000, // 7 moves
        /* J1 off */ 0,
        /* J2 off */ 0,
        /* J3 off */ 0,
        /* J4 off */ 0,
        /* J5 idx103 */ 0x00180100380100000000000000000000, // 7 moves
        /* J6 idx104 */ 0x00380280780200000000000000000000, // 10 moves
        /* J7 idx105 */ 0x00780500F00400000000000000000000, // 11 moves
        /* J8 idx106 */ 0x00F00A01E00800000000000000000000, // 11 moves
        /* J9 idx107 */ 0x01E01403C01000000000000000000000, // 11 moves
        /* J10 idx108 */ 0x01C02807802000000000000000000000, // 10 moves
        /* J11 idx109 */ 0x01801007004000000000000000000000, // 7 moves
        /* K1 off */ 0,
        /* K2 off */ 0,
        /* K3 off */ 0,
        /* K4 off */ 0,
        /* K5 off */ 0,
        /* K6 idx115 */ 0x00100380100000000000000000000000, // 5 moves
        /* K7 idx116 */ 0x00280780200000000000000000000000, // 7 moves
        /* K8 idx117 */ 0x00500F00400000000000000000000000, // 7 moves
        /* K9 idx118 */ 0x00A01E00800000000000000000000000, // 7 moves
        /* K10 idx119 */ 0x01403C01000000000000000000000000, // 7 moves
        /* K11 idx120 */ 0x00803802000000000000000000000000, // 5 moves
    ];

    const BISHOP_MAGICS: [Magic; 121] = [
        /* A1  idx   0 */
        Magic {
            mask: 0x00000000000000000020040080000000,
            magic: 0x61804018804001042404300400020000,
            shift: 124,
            offset: 0,
        },
        /* A2  idx   1 */
        Magic {
            mask: 0x00000000000000000440080100000000,
            magic: 0x04084200400045000088001041004000,
            shift: 124,
            offset: 16,
        },
        /* A3  idx   2 */
        Magic {
            mask: 0x00000000000000008801002000040000,
            magic: 0x14041002820480300424008001002000,
            shift: 124,
            offset: 32,
        },
        /* A4  idx   3 */
        Magic {
            mask: 0x00000000000001102004000008000000,
            magic: 0x00805001021008040280424018110000,
            shift: 124,
            offset: 48,
        },
        /* A5  idx   4 */
        Magic {
            mask: 0x00000000000022040080000010000000,
            magic: 0x05401180205800081010050800040000,
            shift: 124,
            offset: 64,
        },
        /* A6  idx   5 */
        Magic {
            mask: 0x00000000000044080100000020000000,
            magic: 0x00220448100140220020040101010000,
            shift: 124,
            offset: 84,
        },
        /* A7  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 100,
        },
        /* A8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 100,
        },
        /* A9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 100,
        },
        /* A10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 100,
        },
        /* A11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 100,
        },
        /* B1  idx  11 */
        Magic {
            mask: 0x00000000000000000040080100200000,
            magic: 0x04040410214081005018040010084000,
            shift: 124,
            offset: 100,
        },
        /* B2  idx  12 */
        Magic {
            mask: 0x00000000000000000880100200400000,
            magic: 0x00410114002010882142200440402000,
            shift: 124,
            offset: 116,
        },
        /* B3  idx  13 */
        Magic {
            mask: 0x00000000000000011020040080800000,
            magic: 0x00808818814838020840240101020000,
            shift: 123,
            offset: 132,
        },
        /* B4  idx  14 */
        Magic {
            mask: 0x00000000000002204008010100000000,
            magic: 0x10102830413000801102080081044000,
            shift: 123,
            offset: 164,
        },
        /* B5  idx  15 */
        Magic {
            mask: 0x00000000000044080100202000000000,
            magic: 0x0318041400a40204850840c100040000,
            shift: 123,
            offset: 196,
        },
        /* B6  idx  16 */
        Magic {
            mask: 0x00000000000088100200404000000000,
            magic: 0x101410c5000940024220100400c01000,
            shift: 123,
            offset: 228,
        },
        /* B7  idx  17 */
        Magic {
            mask: 0x00000000000110200400808000000000,
            magic: 0x041308a002440301010c240084801000,
            shift: 123,
            offset: 260,
        },
        /* B8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 292,
        },
        /* B9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 292,
        },
        /* B10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 292,
        },
        /* B11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 292,
        },
        /* C1  idx  22 */
        Magic {
            mask: 0x00000000000000000880100200400801,
            magic: 0x00a0a01004245412400808a202201000,
            shift: 123,
            offset: 292,
        },
        /* C2  idx  23 */
        Magic {
            mask: 0x00000000000000011020040080801002,
            magic: 0x0022410a08340114c004818128404000,
            shift: 122,
            offset: 324,
        },
        /* C3  idx  24 */
        Magic {
            mask: 0x00000000000002204008010101002004,
            magic: 0x0282a51010028c0010884a1005820000,
            shift: 122,
            offset: 388,
        },
        /* C4  idx  25 */
        Magic {
            mask: 0x00000000000044080100202020004008,
            magic: 0x008d1840810488410d21054224090000,
            shift: 122,
            offset: 452,
        },
        /* C5  idx  26 */
        Magic {
            mask: 0x00000000000088100200404040008010,
            magic: 0x181050a41815042211020e4088050000,
            shift: 122,
            offset: 516,
        },
        /* C6  idx  27 */
        Magic {
            mask: 0x00000000000110200400808080010020,
            magic: 0x0080e5445214041a02100804c8430000,
            shift: 122,
            offset: 580,
        },
        /* C7  idx  28 */
        Magic {
            mask: 0x00000000000220400801010101002004,
            magic: 0x0c485a0881a424220c4a408111a40000,
            shift: 122,
            offset: 644,
        },
        /* C8  idx  29 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x00c401340166245100e129158c304000,
            shift: 122,
            offset: 708,
        },
        /* C9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 772,
        },
        /* C10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 772,
        },
        /* C11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 772,
        },
        /* D1  idx  33 */
        Magic {
            mask: 0x00000000000000011020040080801002,
            magic: 0x008c2401880403c40184c106428c0400,
            shift: 123,
            offset: 772,
        },
        /* D2  idx  34 */
        Magic {
            mask: 0x00000000000002204008010101002004,
            magic: 0x02a1140685040d34208a00298a0c2000,
            shift: 122,
            offset: 804,
        },
        /* D3  idx  35 */
        Magic {
            mask: 0x00000000000044080100202020004008,
            magic: 0x00908852309101d222d4809d30c80000,
            shift: 122,
            offset: 868,
        },
        /* D4  idx  36 */
        Magic {
            mask: 0x00000000000088100200404040008010,
            magic: 0x082522c0d5c0b8cc194c5e62cda12000,
            shift: 122,
            offset: 932,
        },
        /* D5  idx  37 */
        Magic {
            mask: 0x00000000000110200400808080010020,
            magic: 0x0801c8da9a4c7e6ec4baf070e6091000,
            shift: 121,
            offset: 996,
        },
        /* D6  idx  38 */
        Magic {
            mask: 0x00000000000220400801010101002004,
            magic: 0x0013ca6a4b3af2dbe44bf56b3e7cd800,
            shift: 121,
            offset: 1124,
        },
        /* D7  idx  39 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x0606b29cd166be4eecdbf1cadd5ee800,
            shift: 121,
            offset: 1252,
        },
        /* D8  idx  40 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x04467d0cf0c4b2ca5ceab562c56a3000,
            shift: 121,
            offset: 1380,
        },
        /* D9  idx  41 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x0113c2ce526cc6e144ea1b36aaac5000,
            shift: 121,
            offset: 1508,
        },
        /* D10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1636,
        },
        /* D11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1636,
        },
        /* E1  idx  44 */
        Magic {
            mask: 0x00000000000002204008010101002004,
            magic: 0x010c28308434a53821049210c4224000,
            shift: 122,
            offset: 1636,
        },
        /* E2  idx  45 */
        Magic {
            mask: 0x00000000000044080100202020004008,
            magic: 0x0092491148e65842188da50d03611000,
            shift: 122,
            offset: 1700,
        },
        /* E3  idx  46 */
        Magic {
            mask: 0x00000000000088100200404040008010,
            magic: 0x003180c430291e66c62ca16cc8c3000,
            shift: 122,
            offset: 1764,
        },
        /* E4  idx  47 */
        Magic {
            mask: 0x00000000000110200400808080010020,
            magic: 0x048cdcd4cc9adaca4e3ada5ecd3b1000,
            shift: 121,
            offset: 1828,
        },
        /* E5  idx  48 */
        Magic {
            mask: 0x00000000000220400801010101002004,
            magic: 0x04825ece6cc6be7a0cdc8669ae494000,
            shift: 121,
            offset: 1956,
        },
        /* E6  idx  49 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x054caea792dceede2ee6bc4add1e5800,
            shift: 120,
            offset: 2084,
        },
        /* E7  idx  50 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x0d41e7dcdbc0baca1dc73e52ee3e3000,
            shift: 120,
            offset: 2340,
        },
        /* E8  idx  51 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x03a67aebe5cc4ca74e6efab6ac4b9000,
            shift: 120,
            offset: 2596,
        },
        /* E9  idx  52 */
        Magic {
            mask: 0x00000000002204008010101010020040,
            magic: 0x053075aebe64f2abce6afca2c52aa000,
            shift: 120,
            offset: 2852,
        },
        /* E10 idx  53 */
        Magic {
            mask: 0x00000000004408010020202020040080,
            magic: 0x01138beaa4ac84c146ea1706ca25000,
            shift: 121,
            offset: 3108,
        },
        /* E11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 3236,
        },
        /* F1  idx  55 */
        Magic {
            mask: 0x00000000000044080100202020004008,
            magic: 0x028c253810c950529124408c02c01000,
            shift: 122,
            offset: 3236,
        },
        /* F2  idx  56 */
        Magic {
            mask: 0x00000000000088100200404040008010,
            magic: 0x029402542183c21a48c9a38431102000,
            shift: 122,
            offset: 3300,
        },
        /* F3  idx  57 */
        Magic {
            mask: 0x00000000000110200400808080010020,
            magic: 0x0188c6c518aa72da1aca31cd330b2000,
            shift: 121,
            offset: 3364,
        },
        /* F4  idx  58 */
        Magic {
            mask: 0x00000000000220400801010101002004,
            magic: 0x0c0ea9b4a44ebeee0cd6b5b5b4ad1000,
            shift: 121,
            offset: 3492,
        },
        /* F5  idx  59 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x0611a5bba6dcfedee69ef6caeedad000,
            shift: 120,
            offset: 3620,
        },
        /* F6  idx  60 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x01416bd9dbccd6de0e6af4b4eb2b7000,
            shift: 120,
            offset: 3876,
        },
        /* F7  idx  61 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x0696f5baa728fede64cee6daeae31000,
            shift: 120,
            offset: 4132,
        },
        /* F8  idx  62 */
        Magic {
            mask: 0x00000000002204008010101010020040,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 121,
            offset: 4388,
        },
        /* F9  idx  63 */
        Magic {
            mask: 0x00000000004408010020202020040080,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 121,
            offset: 4516,
        },
        /* F10 idx  64 */
        Magic {
            mask: 0x00000000008810020040404040080100,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 122,
            offset: 4644,
        },
        /* F11 idx  65 */
        Magic {
            mask: 0x00000000011020040080808080100200,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 122,
            offset: 4708,
        },
        /* G1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 4772,
        },
        /* G2  idx  67 */
        Magic {
            mask: 0x00000000000088100200404040008010,
            magic: 0x008da50d20d188421849150424090000,
            shift: 122,
            offset: 4772,
        },
        /* G3  idx  68 */
        Magic {
            mask: 0x00000000000110200400808080010020,
            magic: 0x018cc6c98aa124aa1aca118eaab68000,
            shift: 121,
            offset: 4836,
        },
        /* G4  idx  69 */
        Magic {
            mask: 0x00000000000220400801010101002004,
            magic: 0x04825eaecac67e720c9c3e4eee3c1000,
            shift: 121,
            offset: 4964,
        },
        /* G5  idx  70 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x03a6fadfe72c4caee4eeeefcaea9a000,
            shift: 120,
            offset: 5092,
        },
        /* G6  idx  71 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x0d41e7d8abc0baca1dc23ebaeaea3000,
            shift: 120,
            offset: 5348,
        },
        /* G7  idx  72 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x052cfade7a24c2eeae6aecaa67caa000,
            shift: 120,
            offset: 5604,
        },
        /* G8  idx  73 */
        Magic {
            mask: 0x00000000002204008010101010020040,
            magic: 0x0481deaee6647e740cdc2a54ee34a000,
            shift: 120,
            offset: 5860,
        },
        /* G9  idx  74 */
        Magic {
            mask: 0x00000000004408010020202020040080,
            magic: 0x0184ca91aa216ca146a81a2daacc4000,
            shift: 121,
            offset: 6116,
        },
        /* G10 idx  75 */
        Magic {
            mask: 0x00000000008810020040404040080100,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 122,
            offset: 6244,
        },
        /* G11 idx  76 */
        Magic {
            mask: 0x00000000011020040080808080100200,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 122,
            offset: 6308,
        },
        /* H1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 6372,
        },
        /* H2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 6372,
        },
        /* H3  idx  79 */
        Magic {
            mask: 0x00000000000110200400808080010020,
            magic: 0x0188c6c8206010da1aca010260ca2000,
            shift: 121,
            offset: 6372,
        },
        /* H4  idx  80 */
        Magic {
            mask: 0x00000000000220400801010101002004,
            magic: 0x04025ea8c40442320c8c0e0cca421000,
            shift: 121,
            offset: 6500,
        },
        /* H5  idx  81 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x02a27adcc92410a646ee8ea6ac493000,
            shift: 121,
            offset: 6628,
        },
        /* H6  idx  82 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x0541e7d825c09aca19c21e3cae6a3000,
            shift: 121,
            offset: 6756,
        },
        /* H7  idx  83 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x01247ade6e2482ee4e6ae29a67cb1000,
            shift: 121,
            offset: 6884,
        },
        /* H8  idx  84 */
        Magic {
            mask: 0x00000000002204008010101010020040,
            magic: 0x0481deae66645e740cdc1e10ee2d2000,
            shift: 121,
            offset: 7012,
        },
        /* H9  idx  85 */
        Magic {
            mask: 0x00000000004408010020202020040080,
            magic: 0x0100ca91aa2168a14428122daac22000,
            shift: 121,
            offset: 7140,
        },
        /* H10 idx  86 */
        Magic {
            mask: 0x00000000008810020040404040080100,
            magic: 0x0088a501244180aa18241114ac204000,
            shift: 122,
            offset: 7268,
        },
        /* H11 idx  87 */
        Magic {
            mask: 0x00000000011020040080808080100200,
            magic: 0x01082300853048922048110a20420000,
            shift: 122,
            offset: 7332,
        },
        /* I1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 7396,
        },
        /* I2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 7396,
        },
        /* I3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 7396,
        },
        /* I4  idx  91 */
        Magic {
            mask: 0x00000000000220400801010101002004,
            magic: 0x040214a00404023204840e04240a1000,
            shift: 122,
            offset: 7396,
        },
        /* I5  idx  92 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x00a270dc8124102606ae06a62c453000,
            shift: 122,
            offset: 7460,
        },
        /* I6  idx  93 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x014167d021c090ca09c2161ca62a3000,
            shift: 122,
            offset: 7524,
        },
        /* I7  idx  94 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x01243ade4224822e0e6ae11a27cb1000,
            shift: 122,
            offset: 7588,
        },
        /* I8  idx  95 */
        Magic {
            mask: 0x00000000002204008010101010020040,
            magic: 0x04819eae42645a740cdc1610662d2000,
            shift: 122,
            offset: 7652,
        },
        /* I9  idx  96 */
        Magic {
            mask: 0x00000000004408010020202020040080,
            magic: 0x01008a912a2168214428111daac22000,
            shift: 122,
            offset: 7716,
        },
        /* I10 idx  97 */
        Magic {
            mask: 0x00000000008810020040404040080100,
            magic: 0x008825012041802a1824110ca4204000,
            shift: 123,
            offset: 7780,
        },
        /* I11 idx  98 */
        Magic {
            mask: 0x00000000011020040080808080100200,
            magic: 0x01081100813048122048110220420000,
            shift: 123,
            offset: 7812,
        },
        /* J1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 7844,
        },
        /* J2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 7844,
        },
        /* J3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 7844,
        },
        /* J4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 7844,
        },
        /* J5  idx 103 */
        Magic {
            mask: 0x00000000000440801002020202004008,
            magic: 0x0022301001241006022e022204413000,
            shift: 123,
            offset: 7844,
        },
        /* J6  idx 104 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x014107102140900a09421210222a3000,
            shift: 123,
            offset: 7876,
        },
        /* J7  idx 105 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x0124125e4224802e064ae01a224b1000,
            shift: 123,
            offset: 7908,
        },
        /* J8  idx 106 */
        Magic {
            mask: 0x00000000002204008010101010020040,
            magic: 0x04811eae42245a340cdc121026252000,
            shift: 123,
            offset: 7940,
        },
        /* J9  idx 107 */
        Magic {
            mask: 0x00000000004408010020202020040080,
            magic: 0x010012912a2168214228111522c22000,
            shift: 123,
            offset: 7972,
        },
        /* J10 idx 108 */
        Magic {
            mask: 0x00000000008810020040404040080100,
            magic: 0x008811012041802a1024110aa0204000,
            shift: 124,
            offset: 8004,
        },
        /* J11 idx 109 */
        Magic {
            mask: 0x00000000011020040080808080100200,
            magic: 0x01081100811048122048110220420000,
            shift: 124,
            offset: 8020,
        },
        /* K1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 8036,
        },
        /* K2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 8036,
        },
        /* K3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 8036,
        },
        /* K4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 8036,
        },
        /* K5  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 8036,
        },
        /* K6  idx 115 */
        Magic {
            mask: 0x00000000000881002004040404008010,
            magic: 0x00410110214010020142101022023000,
            shift: 124,
            offset: 8036,
        },
        /* K7  idx 116 */
        Magic {
            mask: 0x00000000001102004008080808010020,
            magic: 0x0124111e4224802a024ae01a22031000,
            shift: 124,
            offset: 8052,
        },
        /* K8  idx 117 */
        Magic {
            mask: 0x00000000002204008010101010020040,
            magic: 0x04811eae42241a340cdc121022252000,
            shift: 124,
            offset: 8068,
        },
        /* K9  idx 118 */
        Magic {
            mask: 0x00000000004408010020202020040080,
            magic: 0x010012912a2128214228111522422000,
            shift: 124,
            offset: 8084,
        },
        /* K10 idx 119 */
        Magic {
            mask: 0x00000000008810020040404040080100,
            magic: 0x008811012041802a1024110620204000,
            shift: 125,
            offset: 8100,
        },
        /* K11 idx 120 */
        Magic {
            mask: 0x00000000011020040080808080100200,
            magic: 0x01081100811048122048110220420000,
            shift: 125,
            offset: 8108,
        },
    ];

    const ROOK_R_MAGICS: [Magic; 121] = [
        /* A1  idx   0 */
        Magic {
            mask: 0x000000000000000000000000000007C0,
            magic: 0x1f02148408a110a18431055018a38100,
            shift: 123,
            offset: 0,
        },
        /* A2  idx   1 */
        Magic {
            mask: 0x0000000000000000000000000000F800,
            magic: 0x07de0255c4d0a138902048500c563800,
            shift: 123,
            offset: 32,
        },
        /* A3  idx   2 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc4214488b0a99182410a6cc662400,
            shift: 123,
            offset: 64,
        },
        /* A4  idx   3 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f1141444d32a4a82410d4ea3d0800,
            shift: 123,
            offset: 96,
        },
        /* A5  idx   4 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fa2514d868995a91448dbbbba2800,
            shift: 123,
            offset: 128,
        },
        /* A6  idx   5 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003e8a4a4b2a4aa92212959ea2fa800,
            shift: 123,
            offset: 160,
        },
        /* A7  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* B1  idx  11 */
        Magic {
            mask: 0x000000000000000000000000000007C0,
            magic: 0x1f0214821c3222a4d0411a141aa32200,
            shift: 123,
            offset: 192,
        },
        /* B2  idx  12 */
        Magic {
            mask: 0x0000000000000000000000000000F800,
            magic: 0x07da290cb646eaec82245a90962b4800,
            shift: 123,
            offset: 224,
        },
        /* B3  idx  13 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc462aecccdaca51249b5aeccb0c00,
            shift: 123,
            offset: 256,
        },
        /* B4  idx  14 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 288,
        },
        /* B5  idx  15 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 320,
        },
        /* B6  idx  16 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 352,
        },
        /* B7  idx  17 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 384,
        },
        /* B8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* C1  idx  22 */
        Magic {
            mask: 0x000000000000000000000000000007C0,
            magic: 0x1f0214821c3222a4d0411a141aa32200,
            shift: 123,
            offset: 416,
        },
        /* C2  idx  23 */
        Magic {
            mask: 0x0000000000000000000000000000F800,
            magic: 0x07da290cb646eaec82245a90962b4800,
            shift: 123,
            offset: 448,
        },
        /* C3  idx  24 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc462aecccdaca51249b5aeccb0c00,
            shift: 123,
            offset: 480,
        },
        /* C4  idx  25 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 512,
        },
        /* C5  idx  26 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 544,
        },
        /* C6  idx  27 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 576,
        },
        /* C7  idx  28 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 608,
        },
        /* C8  idx  29 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 640,
        },
        /* C9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* C10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* C11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* D1  idx  33 */
        Magic {
            mask: 0x000000000000000000000000000007C0,
            magic: 0x1f0214821c3222a4d0411a141aa32200,
            shift: 123,
            offset: 672,
        },
        /* D2  idx  34 */
        Magic {
            mask: 0x0000000000000000000000000000F800,
            magic: 0x07da290cb646eaec82245a90962b4800,
            shift: 123,
            offset: 704,
        },
        /* D3  idx  35 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc462aecccdaca51249b5aeccb0c00,
            shift: 123,
            offset: 736,
        },
        /* D4  idx  36 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 768,
        },
        /* D5  idx  37 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 800,
        },
        /* D6  idx  38 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 832,
        },
        /* D7  idx  39 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 864,
        },
        /* D8  idx  40 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 896,
        },
        /* D9  idx  41 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 928,
        },
        /* D10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 960,
        },
        /* D11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 960,
        },
        /* E1  idx  44 */
        Magic {
            mask: 0x000000000000000000000000000007C0,
            magic: 0x1f0214821c3222a4d0411a141aa32200,
            shift: 123,
            offset: 960,
        },
        /* E2  idx  45 */
        Magic {
            mask: 0x0000000000000000000000000000F800,
            magic: 0x07da290cb646eaec82245a90962b4800,
            shift: 123,
            offset: 992,
        },
        /* E3  idx  46 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc462aecccdaca51249b5aeccb0c00,
            shift: 123,
            offset: 1024,
        },
        /* E4  idx  47 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 1056,
        },
        /* E5  idx  48 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 1088,
        },
        /* E6  idx  49 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 1120,
        },
        /* E7  idx  50 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 1152,
        },
        /* E8  idx  51 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 1184,
        },
        /* E9  idx  52 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 1216,
        },
        /* E10 idx  53 */
        Magic {
            mask: 0x00000000000000000003E00000000000,
            magic: 0x000003fabaaa4cae4aa2eb6caaaa800,
            shift: 123,
            offset: 1248,
        },
        /* E11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1280,
        },
        /* F1  idx  55 */
        Magic {
            mask: 0x000000000000000000000000000003E0,
            magic: 0x0f810a410e191150c2208d0c0c511000,
            shift: 123,
            offset: 1280,
        },
        /* F2  idx  56 */
        Magic {
            mask: 0x0000000000000000000000000000F800,
            magic: 0x07da290cb646eaec82245a90962b4800,
            shift: 123,
            offset: 1312,
        },
        /* F3  idx  57 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc462aecccdaca51249b5aeccb0c00,
            shift: 123,
            offset: 1344,
        },
        /* F4  idx  58 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 1376,
        },
        /* F5  idx  59 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 1408,
        },
        /* F6  idx  60 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 1440,
        },
        /* F7  idx  61 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 1472,
        },
        /* F8  idx  62 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 1504,
        },
        /* F9  idx  63 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 1536,
        },
        /* F10 idx  64 */
        Magic {
            mask: 0x00000000000000000003E00000000000,
            magic: 0x000003fabaaa4cae4aa2eb6caaaa800,
            shift: 123,
            offset: 1568,
        },
        /* F11 idx  65 */
        Magic {
            mask: 0x00000000000000000000000000000000,
            magic: 0x0,
            shift: 128,
            offset: 1600,
        },
        /* G1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1600,
        },
        /* G2  idx  67 */
        Magic {
            mask: 0x0000000000000000000000000000F800,
            magic: 0x07da290cb646eaec82245a90962b4800,
            shift: 123,
            offset: 1600,
        },
        /* G3  idx  68 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc462aecccdaca51249b5aeccb0c00,
            shift: 123,
            offset: 1632,
        },
        /* G4  idx  69 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 1664,
        },
        /* G5  idx  70 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 1696,
        },
        /* G6  idx  71 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 1728,
        },
        /* G7  idx  72 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 1760,
        },
        /* G8  idx  73 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 1792,
        },
        /* G9  idx  74 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 1824,
        },
        /* G10 idx  75 */
        Magic {
            mask: 0x00000000000000000003E00000000000,
            magic: 0x000003fabaaa4cae4aa2eb6caaaa800,
            shift: 123,
            offset: 1856,
        },
        /* G11 idx  76 */
        Magic {
            mask: 0x00000000000000000000000000000000,
            magic: 0x0,
            shift: 128,
            offset: 1888,
        },
        /* H1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1888,
        },
        /* H2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1888,
        },
        /* H3  idx  79 */
        Magic {
            mask: 0x000000000000000000000000001F0000,
            magic: 0x00fc462aecccdaca51249b5aeccb0c00,
            shift: 123,
            offset: 1888,
        },
        /* H4  idx  80 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 1920,
        },
        /* H5  idx  81 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 1952,
        },
        /* H6  idx  82 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 1984,
        },
        /* H7  idx  83 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 2016,
        },
        /* H8  idx  84 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 2048,
        },
        /* H9  idx  85 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 2080,
        },
        /* H10 idx  86 */
        Magic {
            mask: 0x00000000000000000003E00000000000,
            magic: 0x000003fabaaa4cae4aa2eb6caaaa800,
            shift: 123,
            offset: 2112,
        },
        /* H11 idx  87 */
        Magic {
            mask: 0x00000000000000000000000000000000,
            magic: 0x0,
            shift: 128,
            offset: 2144,
        },
        /* I1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2144,
        },
        /* I2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2144,
        },
        /* I3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2144,
        },
        /* I4  idx  91 */
        Magic {
            mask: 0x00000000000000000000000003E00000,
            magic: 0x003f0aa7bb34d3b14512535ae7aa2800,
            shift: 123,
            offset: 2144,
        },
        /* I5  idx  92 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 2176,
        },
        /* I6  idx  93 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 2208,
        },
        /* I7  idx  94 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 2240,
        },
        /* I8  idx  95 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 2272,
        },
        /* I9  idx  96 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 2304,
        },
        /* I10 idx  97 */
        Magic {
            mask: 0x00000000000000000003E00000000000,
            magic: 0x000003fabaaa4cae4aa2eb6caaaa800,
            shift: 123,
            offset: 2336,
        },
        /* I11 idx  98 */
        Magic {
            mask: 0x00000000000000000000000000000000,
            magic: 0x0,
            shift: 128,
            offset: 2368,
        },
        /* J1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2368,
        },
        /* J2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2368,
        },
        /* J3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2368,
        },
        /* J4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2368,
        },
        /* J5  idx 103 */
        Magic {
            mask: 0x0000000000000000000000007C000000,
            magic: 0x000fc26bb4dc8fa0d4c94cdbbf3a4800,
            shift: 123,
            offset: 2368,
        },
        /* J6  idx 104 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 2400,
        },
        /* J7  idx 105 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 2432,
        },
        /* J8  idx 106 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 2464,
        },
        /* J9  idx 107 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 2496,
        },
        /* J10 idx 108 */
        Magic {
            mask: 0x00000000000000000003E00000000000,
            magic: 0x000003fabaaa4cae4aa2eb6caaaa800,
            shift: 123,
            offset: 2528,
        },
        /* J11 idx 109 */
        Magic {
            mask: 0x00000000000000000000000000000000,
            magic: 0x0,
            shift: 128,
            offset: 2560,
        },
        /* K1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2560,
        },
        /* K2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2560,
        },
        /* K3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2560,
        },
        /* K4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2560,
        },
        /* K5  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2560,
        },
        /* K6  idx 115 */
        Magic {
            mask: 0x00000000000000000000000F80000000,
            magic: 0x0003eaee8b6a9ea51622aa5eea6b4800,
            shift: 123,
            offset: 2560,
        },
        /* K7  idx 116 */
        Magic {
            mask: 0x0000000000000000000000F800000000,
            magic: 0x0000faeecb2cbabaaa14ca9faeeea800,
            shift: 123,
            offset: 2592,
        },
        /* K8  idx 117 */
        Magic {
            mask: 0x000000000000000000000F8000000000,
            magic: 0x00003eefbe6ebef6ea2296bdaeaeb800,
            shift: 123,
            offset: 2624,
        },
        /* K9  idx 118 */
        Magic {
            mask: 0x00000000000000000000F80000000000,
            magic: 0x00000ffbeaaa4caeea6aeeb6caaea800,
            shift: 123,
            offset: 2656,
        },
        /* K10 idx 119 */
        Magic {
            mask: 0x00000000000000000003E00000000000,
            magic: 0x000003fabaaa4cae4aa2eb6caaaa800,
            shift: 123,
            offset: 2688,
        },
        /* K11 idx 120 */
        Magic {
            mask: 0x00000000000000000000000000000000,
            magic: 0x0,
            shift: 128,
            offset: 2720,
        },
    ];

    const ROOK_C_MAGICS: [Magic; 121] = [
        /* A1  idx   0 */
        Magic {
            mask: 0x000040080100202020004000,
            magic: 0x0114002450005080c502120c43041000,
            shift: 123,
            offset: 0,
        },
        /* A2  idx   1 */
        Magic {
            mask: 0x000080100200404040008000,
            magic: 0x03a435aeeb5daeaece224a5eea3b2000,
            shift: 123,
            offset: 32,
        },
        /* A3  idx   2 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 64,
        },
        /* A4  idx   3 */
        Magic {
            mask: 0x000220400801010101002000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 96,
        },
        /* A5  idx   4 */
        Magic {
            mask: 0x000440801002020202004000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 128,
        },
        /* A6  idx   5 */
        Magic {
            mask: 0x000881002004040404008000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 160,
        },
        /* A7  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* B1  idx  11 */
        Magic {
            mask: 0x000040080100202020000000,
            magic: 0x0114002450005080c502120c43041000,
            shift: 123,
            offset: 192,
        },
        /* B2  idx  12 */
        Magic {
            mask: 0x000080100200404040008000,
            magic: 0x03a435aeeb5daeaece224a5eea3b2000,
            shift: 123,
            offset: 224,
        },
        /* B3  idx  13 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 256,
        },
        /* B4  idx  14 */
        Magic {
            mask: 0x000220400801010101002000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 288,
        },
        /* B5  idx  15 */
        Magic {
            mask: 0x000440801002020202004000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 320,
        },
        /* B6  idx  16 */
        Magic {
            mask: 0x000881002004040404008000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 352,
        },
        /* B7  idx  17 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 384,
        },
        /* B8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* C1  idx  22 */
        Magic {
            mask: 0x000040080100202020000000,
            magic: 0x0114002450005080c502120c43041000,
            shift: 123,
            offset: 416,
        },
        /* C2  idx  23 */
        Magic {
            mask: 0x000080100200404040000000,
            magic: 0x03a435aeeb5daeaece224a5eea3b2000,
            shift: 123,
            offset: 448,
        },
        /* C3  idx  24 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 480,
        },
        /* C4  idx  25 */
        Magic {
            mask: 0x000220400801010101002000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 512,
        },
        /* C5  idx  26 */
        Magic {
            mask: 0x000440801002020202004000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 544,
        },
        /* C6  idx  27 */
        Magic {
            mask: 0x000881002004040404008000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 576,
        },
        /* C7  idx  28 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 608,
        },
        /* C8  idx  29 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 640,
        },
        /* C9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* C10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* C11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* D1  idx  33 */
        Magic {
            mask: 0x000040080100202020000000,
            magic: 0x0114002450005080c502120c43041000,
            shift: 123,
            offset: 672,
        },
        /* D2  idx  34 */
        Magic {
            mask: 0x000080100200404040000000,
            magic: 0x03a435aeeb5daeaece224a5eea3b2000,
            shift: 123,
            offset: 704,
        },
        /* D3  idx  35 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 736,
        },
        /* D4  idx  36 */
        Magic {
            mask: 0x000220400801010101000000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 768,
        },
        /* D5  idx  37 */
        Magic {
            mask: 0x000440801002020202004000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 800,
        },
        /* D6  idx  38 */
        Magic {
            mask: 0x000881002004040404008000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 832,
        },
        /* D7  idx  39 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 864,
        },
        /* D8  idx  40 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 896,
        },
        /* D9  idx  41 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 928,
        },
        /* D10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 960,
        },
        /* D11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 960,
        },
        /* E1  idx  44 */
        Magic {
            mask: 0x000040080100202020000000,
            magic: 0x0114002450005080c502120c43041000,
            shift: 123,
            offset: 960,
        },
        /* E2  idx  45 */
        Magic {
            mask: 0x000080100200404040000000,
            magic: 0x03a435aeeb5daeaece224a5eea3b2000,
            shift: 123,
            offset: 992,
        },
        /* E3  idx  46 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 1024,
        },
        /* E4  idx  47 */
        Magic {
            mask: 0x000220400801010101000000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 1056,
        },
        /* E5  idx  48 */
        Magic {
            mask: 0x000440801002020202000000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 1088,
        },
        /* E6  idx  49 */
        Magic {
            mask: 0x000881002004040404008000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 1120,
        },
        /* E7  idx  50 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 1152,
        },
        /* E8  idx  51 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 1184,
        },
        /* E9  idx  52 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 1216,
        },
        /* E10 idx  53 */
        Magic {
            mask: 0x008810002040404040080000,
            magic: 0x01141128410881510484510620850000,
            shift: 123,
            offset: 1248,
        },
        /* E11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1280,
        },
        /* F1  idx  55 */
        Magic {
            mask: 0x000040080100202020000000,
            magic: 0x0114002450005080c502120c43041000,
            shift: 123,
            offset: 1280,
        },
        /* F2  idx  56 */
        Magic {
            mask: 0x000080100200404040000000,
            magic: 0x03a435aeeb5daeaece224a5eea3b2000,
            shift: 123,
            offset: 1312,
        },
        /* F3  idx  57 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 1344,
        },
        /* F4  idx  58 */
        Magic {
            mask: 0x000220400801010101000000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 1376,
        },
        /* F5  idx  59 */
        Magic {
            mask: 0x000440801002020202000000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 1408,
        },
        /* F6  idx  60 */
        Magic {
            mask: 0x000881002004040404000000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 1440,
        },
        /* F7  idx  61 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 1472,
        },
        /* F8  idx  62 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 1504,
        },
        /* F9  idx  63 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 1536,
        },
        /* F10 idx  64 */
        Magic {
            mask: 0x008810002040404040080000,
            magic: 0x01141128410881510484510620850000,
            shift: 123,
            offset: 1568,
        },
        /* F11 idx  65 */
        Magic {
            mask: 0x011020004080808080100000,
            magic: 0x00a8a104210440a42448450a20430000,
            shift: 123,
            offset: 1600,
        },
        /* G1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1632,
        },
        /* G2  idx  67 */
        Magic {
            mask: 0x000080100200404040000000,
            magic: 0x03a435aeeb5daeaece224a5eea3b2000,
            shift: 123,
            offset: 1632,
        },
        /* G3  idx  68 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 1664,
        },
        /* G4  idx  69 */
        Magic {
            mask: 0x000220400801010101000000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 1696,
        },
        /* G5  idx  70 */
        Magic {
            mask: 0x000440801002020202000000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 1728,
        },
        /* G6  idx  71 */
        Magic {
            mask: 0x000881002004040404000000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 1760,
        },
        /* G7  idx  72 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 1792,
        },
        /* G8  idx  73 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 1824,
        },
        /* G9  idx  74 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 1856,
        },
        /* G10 idx  75 */
        Magic {
            mask: 0x008810002040404040080000,
            magic: 0x01141128410881510484510620850000,
            shift: 123,
            offset: 1888,
        },
        /* G11 idx  76 */
        Magic {
            mask: 0x011020004080808080100000,
            magic: 0x00a8a104210440a42448450a20430000,
            shift: 123,
            offset: 1920,
        },
        /* H1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1952,
        },
        /* H2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1952,
        },
        /* H3  idx  79 */
        Magic {
            mask: 0x000110200400808080010000,
            magic: 0x0df2bb6eb6fcaefeeee2d4baeedb1000,
            shift: 123,
            offset: 1952,
        },
        /* H4  idx  80 */
        Magic {
            mask: 0x000220400801010101000000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 1984,
        },
        /* H5  idx  81 */
        Magic {
            mask: 0x000440801002020202000000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 2016,
        },
        /* H6  idx  82 */
        Magic {
            mask: 0x000881002004040404000000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 2048,
        },
        /* H7  idx  83 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 2080,
        },
        /* H8  idx  84 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 2112,
        },
        /* H9  idx  85 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 2144,
        },
        /* H10 idx  86 */
        Magic {
            mask: 0x008810002040404040080000,
            magic: 0x01141128410881510484510620850000,
            shift: 123,
            offset: 2176,
        },
        /* H11 idx  87 */
        Magic {
            mask: 0x011020004080808080100000,
            magic: 0x00a8a104210440a42448450a20430000,
            shift: 123,
            offset: 2208,
        },
        /* I1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2240,
        },
        /* I2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2240,
        },
        /* I3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2240,
        },
        /* I4  idx  91 */
        Magic {
            mask: 0x000220400801010101000000,
            magic: 0x0f8aebeebeaed6deeeb69ef6caeed000,
            shift: 123,
            offset: 2240,
        },
        /* I5  idx  92 */
        Magic {
            mask: 0x000440801002020202000000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 2272,
        },
        /* I6  idx  93 */
        Magic {
            mask: 0x000881002004040404000000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 2304,
        },
        /* I7  idx  94 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 2336,
        },
        /* I8  idx  95 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 2368,
        },
        /* I9  idx  96 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 2400,
        },
        /* I10 idx  97 */
        Magic {
            mask: 0x008810002040404040080000,
            magic: 0x01141128410881510484510620850000,
            shift: 123,
            offset: 2432,
        },
        /* I11 idx  98 */
        Magic {
            mask: 0x011020004080808080100000,
            magic: 0x00a8a104210440a42448450a20430000,
            shift: 123,
            offset: 2464,
        },
        /* J1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J5  idx 103 */
        Magic {
            mask: 0x000440801002020202000000,
            magic: 0x0b69efaeeeaeaadeee9efadaeaeaa000,
            shift: 123,
            offset: 2496,
        },
        /* J6  idx 104 */
        Magic {
            mask: 0x000881002004040404000000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 2528,
        },
        /* J7  idx 105 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 2560,
        },
        /* J8  idx 106 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 2592,
        },
        /* J9  idx 107 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 2624,
        },
        /* J10 idx 108 */
        Magic {
            mask: 0x008810002040404040080000,
            magic: 0x01141128410881510484510620850000,
            shift: 123,
            offset: 2656,
        },
        /* J11 idx 109 */
        Magic {
            mask: 0x011020004080808080100000,
            magic: 0x00a8a104210440a42448450a20430000,
            shift: 123,
            offset: 2688,
        },
        /* K1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K5  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K6  idx 115 */
        Magic {
            mask: 0x000881002004040404000000,
            magic: 0x0d41e7dcdbc6baee1de6efbaeaeb2000,
            shift: 123,
            offset: 2720,
        },
        /* K7  idx 116 */
        Magic {
            mask: 0x001102004008080808010000,
            magic: 0x06aeefaeae2aecdaae2eeeb6caaca000,
            shift: 123,
            offset: 2752,
        },
        /* K8  idx 117 */
        Magic {
            mask: 0x002204008010101010020000,
            magic: 0x0d48aaeea24a12ca924ca54eaca38000,
            shift: 123,
            offset: 2784,
        },
        /* K9  idx 118 */
        Magic {
            mask: 0x004408001020202020040000,
            magic: 0x03a22530812111521224910e20c30000,
            shift: 123,
            offset: 2816,
        },
        /* K10 idx 119 */
        Magic {
            mask: 0x008810002040404040080000,
            magic: 0x01141128410881510484510620850000,
            shift: 123,
            offset: 2848,
        },
        /* K11 idx 120 */
        Magic {
            mask: 0x011020004080808080100000,
            magic: 0x00a8a104210440a42448450a20430000,
            shift: 123,
            offset: 2880,
        },
    ];

    const ROOK_X_MAGICS: [Magic; 121] = [
        /* A1  idx   0 */
        Magic {
            mask: 0x000100080040020010000000,
            magic: 0x030825010c3422114224080e0c521000,
            shift: 123,
            offset: 0,
        },
        /* A2  idx   1 */
        Magic {
            mask: 0x000200100080040020000000,
            magic: 0x0611a4321863a21a48c9a38431102000,
            shift: 123,
            offset: 32,
        },
        /* A3  idx   2 */
        Magic {
            mask: 0x000400200100080040000000,
            magic: 0x0c2246aa18aa72da1aca31cd330b2000,
            shift: 123,
            offset: 64,
        },
        /* A4  idx   3 */
        Magic {
            mask: 0x000800400200100080000000,
            magic: 0x0c0ea9b4a44ebeee0cd6b5b5b4ad1000,
            shift: 123,
            offset: 96,
        },
        /* A5  idx   4 */
        Magic {
            mask: 0x001000800400200100000000,
            magic: 0x0611a5bba6dcfedee69ef6caeedad000,
            shift: 123,
            offset: 128,
        },
        /* A6  idx   5 */
        Magic {
            mask: 0x002001000800400200000000,
            magic: 0x01416bd9dbccd6de0e6af4b4eb2b7000,
            shift: 123,
            offset: 160,
        },
        /* A7  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* A11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 192,
        },
        /* B1  idx  11 */
        Magic {
            mask: 0x000200100080040020000000,
            magic: 0x0611a4321863a21a48c9a38431102000,
            shift: 123,
            offset: 192,
        },
        /* B2  idx  12 */
        Magic {
            mask: 0x000400200100080040000000,
            magic: 0x0c2246aa18aa72da1aca31cd330b2000,
            shift: 123,
            offset: 224,
        },
        /* B3  idx  13 */
        Magic {
            mask: 0x000800400200100080000000,
            magic: 0x0c0ea9b4a44ebeee0cd6b5b5b4ad1000,
            shift: 123,
            offset: 256,
        },
        /* B4  idx  14 */
        Magic {
            mask: 0x001000800400200100000000,
            magic: 0x0611a5bba6dcfedee69ef6caeedad000,
            shift: 123,
            offset: 288,
        },
        /* B5  idx  15 */
        Magic {
            mask: 0x002001000800400200000000,
            magic: 0x01416bd9dbccd6de0e6af4b4eb2b7000,
            shift: 123,
            offset: 320,
        },
        /* B6  idx  16 */
        Magic {
            mask: 0x004002001000800400000000,
            magic: 0x0696f5baa728fede64cee6daeae31000,
            shift: 123,
            offset: 352,
        },
        /* B7  idx  17 */
        Magic {
            mask: 0x008004002001000800000000,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 123,
            offset: 384,
        },
        /* B8  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* B11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 416,
        },
        /* C1  idx  22 */
        Magic {
            mask: 0x000400200100080040000000,
            magic: 0x0c2246aa18aa72da1aca31cd330b2000,
            shift: 123,
            offset: 416,
        },
        /* C2  idx  23 */
        Magic {
            mask: 0x000800400200100080000000,
            magic: 0x0c0ea9b4a44ebeee0cd6b5b5b4ad1000,
            shift: 123,
            offset: 448,
        },
        /* C3  idx  24 */
        Magic {
            mask: 0x001000800400200100000000,
            magic: 0x0611a5bba6dcfedee69ef6caeedad000,
            shift: 123,
            offset: 480,
        },
        /* C4  idx  25 */
        Magic {
            mask: 0x002001000800400200000000,
            magic: 0x01416bd9dbccd6de0e6af4b4eb2b7000,
            shift: 123,
            offset: 512,
        },
        /* C5  idx  26 */
        Magic {
            mask: 0x004002001000800400000000,
            magic: 0x0696f5baa728fede64cee6daeae31000,
            shift: 123,
            offset: 544,
        },
        /* C6  idx  27 */
        Magic {
            mask: 0x008004002001000800000000,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 123,
            offset: 576,
        },
        /* C7  idx  28 */
        Magic {
            mask: 0x010008004002001000000000,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 123,
            offset: 608,
        },
        /* C8  idx  29 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 640,
        },
        /* C9  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* C10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* C11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 672,
        },
        /* D1  idx  33 */
        Magic {
            mask: 0x000800400200100080000000,
            magic: 0x0c0ea9b4a44ebeee0cd6b5b5b4ad1000,
            shift: 123,
            offset: 672,
        },
        /* D2  idx  34 */
        Magic {
            mask: 0x001000800400200100000000,
            magic: 0x0611a5bba6dcfedee69ef6caeedad000,
            shift: 123,
            offset: 704,
        },
        /* D3  idx  35 */
        Magic {
            mask: 0x002001000800400200000000,
            magic: 0x01416bd9dbccd6de0e6af4b4eb2b7000,
            shift: 123,
            offset: 736,
        },
        /* D4  idx  36 */
        Magic {
            mask: 0x004002001000800400000000,
            magic: 0x0696f5baa728fede64cee6daeae31000,
            shift: 123,
            offset: 768,
        },
        /* D5  idx  37 */
        Magic {
            mask: 0x008004002001000800000000,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 123,
            offset: 800,
        },
        /* D6  idx  38 */
        Magic {
            mask: 0x010008004002001000000000,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 123,
            offset: 832,
        },
        /* D7  idx  39 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 864,
        },
        /* D8  idx  40 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 896,
        },
        /* D9  idx  41 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 928,
        },
        /* D10 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 960,
        },
        /* D11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 960,
        },
        /* E1  idx  44 */
        Magic {
            mask: 0x001000800400200100000000,
            magic: 0x0611a5bba6dcfedee69ef6caeedad000,
            shift: 123,
            offset: 960,
        },
        /* E2  idx  45 */
        Magic {
            mask: 0x002001000800400200000000,
            magic: 0x01416bd9dbccd6de0e6af4b4eb2b7000,
            shift: 123,
            offset: 992,
        },
        /* E3  idx  46 */
        Magic {
            mask: 0x004002001000800400000000,
            magic: 0x0696f5baa728fede64cee6daeae31000,
            shift: 123,
            offset: 1024,
        },
        /* E4  idx  47 */
        Magic {
            mask: 0x008004002001000800000000,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 123,
            offset: 1056,
        },
        /* E5  idx  48 */
        Magic {
            mask: 0x010008004002001000000000,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 123,
            offset: 1088,
        },
        /* E6  idx  49 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 1120,
        },
        /* E7  idx  50 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 1152,
        },
        /* E8  idx  51 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 1184,
        },
        /* E9  idx  52 */
        Magic {
            mask: 0x100080040020010000000000,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 123,
            offset: 1216,
        },
        /* E10 idx  53 */
        Magic {
            mask: 0x200100080040020000000000,
            magic: 0x01082300853048922048110a20420000,
            shift: 123,
            offset: 1248,
        },
        /* E11 off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1280,
        },
        /* F1  idx  55 */
        Magic {
            mask: 0x002001000800400200000000,
            magic: 0x01416bd9dbccd6de0e6af4b4eb2b7000,
            shift: 123,
            offset: 1280,
        },
        /* F2  idx  56 */
        Magic {
            mask: 0x004002001000800400000000,
            magic: 0x0696f5baa728fede64cee6daeae31000,
            shift: 123,
            offset: 1312,
        },
        /* F3  idx  57 */
        Magic {
            mask: 0x008004002001000800000000,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 123,
            offset: 1344,
        },
        /* F4  idx  58 */
        Magic {
            mask: 0x010008004002001000000000,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 123,
            offset: 1376,
        },
        /* F5  idx  59 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 1408,
        },
        /* F6  idx  60 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 1440,
        },
        /* F7  idx  61 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 1472,
        },
        /* F8  idx  62 */
        Magic {
            mask: 0x100080040020010000000000,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 123,
            offset: 1504,
        },
        /* F9  idx  63 */
        Magic {
            mask: 0x200100080040020000000000,
            magic: 0x01082300853048922048110a20420000,
            shift: 123,
            offset: 1536,
        },
        /* F10 idx  64 */
        Magic {
            mask: 0x400200100080040000000000,
            magic: 0x008825012041802a1824110ca4204000,
            shift: 123,
            offset: 1568,
        },
        /* F11 idx  65 */
        Magic {
            mask: 0x800400200100080000000000,
            magic: 0x004412501021008a0c24910620460000,
            shift: 123,
            offset: 1600,
        },
        /* G1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1632,
        },
        /* G2  idx  67 */
        Magic {
            mask: 0x004002001000800400000000,
            magic: 0x0696f5baa728fede64cee6daeae31000,
            shift: 123,
            offset: 1632,
        },
        /* G3  idx  68 */
        Magic {
            mask: 0x008004002001000800000000,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 123,
            offset: 1664,
        },
        /* G4  idx  69 */
        Magic {
            mask: 0x010008004002001000000000,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 123,
            offset: 1696,
        },
        /* G5  idx  70 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 1728,
        },
        /* G6  idx  71 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 1760,
        },
        /* G7  idx  72 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 1792,
        },
        /* G8  idx  73 */
        Magic {
            mask: 0x100080040020010000000000,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 123,
            offset: 1824,
        },
        /* G9  idx  74 */
        Magic {
            mask: 0x200100080040020000000000,
            magic: 0x01082300853048922048110a20420000,
            shift: 123,
            offset: 1856,
        },
        /* G10 idx  75 */
        Magic {
            mask: 0x400200100080040000000000,
            magic: 0x008825012041802a1824110ca4204000,
            shift: 123,
            offset: 1888,
        },
        /* G11 idx  76 */
        Magic {
            mask: 0x800400200100080000000000,
            magic: 0x004412501021008a0c24910620460000,
            shift: 123,
            offset: 1920,
        },
        /* H1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1952,
        },
        /* H2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 1952,
        },
        /* H3  idx  79 */
        Magic {
            mask: 0x008004002001000800000000,
            magic: 0x0c0aa3b4c4ceb6ee0caeb3b5b62d2000,
            shift: 123,
            offset: 1952,
        },
        /* H4  idx  80 */
        Magic {
            mask: 0x010008004002001000000000,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 123,
            offset: 1984,
        },
        /* H5  idx  81 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 2016,
        },
        /* H6  idx  82 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 2048,
        },
        /* H7  idx  83 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 2080,
        },
        /* H8  idx  84 */
        Magic {
            mask: 0x100080040020010000000000,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 123,
            offset: 2112,
        },
        /* H9  idx  85 */
        Magic {
            mask: 0x200100080040020000000000,
            magic: 0x01082300853048922048110a20420000,
            shift: 123,
            offset: 2144,
        },
        /* H10 idx  86 */
        Magic {
            mask: 0x400200100080040000000000,
            magic: 0x008825012041802a1824110ca4204000,
            shift: 123,
            offset: 2176,
        },
        /* H11 idx  87 */
        Magic {
            mask: 0x800400200100080000000000,
            magic: 0x004412501021008a0c24910620460000,
            shift: 123,
            offset: 2208,
        },
        /* I1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2240,
        },
        /* I2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2240,
        },
        /* I3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2240,
        },
        /* I4  idx  91 */
        Magic {
            mask: 0x010008004002001000000000,
            magic: 0x0188c5a91ca272ea1aca51a3328e4000,
            shift: 123,
            offset: 2240,
        },
        /* I5  idx  92 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 2272,
        },
        /* I6  idx  93 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 2304,
        },
        /* I7  idx  94 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 2336,
        },
        /* I8  idx  95 */
        Magic {
            mask: 0x100080040020010000000000,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 123,
            offset: 2368,
        },
        /* I9  idx  96 */
        Magic {
            mask: 0x200100080040020000000000,
            magic: 0x01082300853048922048110a20420000,
            shift: 123,
            offset: 2400,
        },
        /* I10 idx  97 */
        Magic {
            mask: 0x400200100080040000000000,
            magic: 0x008825012041802a1824110ca4204000,
            shift: 123,
            offset: 2432,
        },
        /* I11 idx  98 */
        Magic {
            mask: 0x800400200100080000000000,
            magic: 0x004412501021008a0c24910620460000,
            shift: 123,
            offset: 2464,
        },
        /* J1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2496,
        },
        /* J5  idx 103 */
        Magic {
            mask: 0x020010008004002000000000,
            magic: 0x028c2294108bc0da488ca21c32148000,
            shift: 123,
            offset: 2496,
        },
        /* J6  idx 104 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 2528,
        },
        /* J7  idx 105 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 2560,
        },
        /* J8  idx 106 */
        Magic {
            mask: 0x100080040020010000000000,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 123,
            offset: 2592,
        },
        /* J9  idx 107 */
        Magic {
            mask: 0x200100080040020000000000,
            magic: 0x01082300853048922048110a20420000,
            shift: 123,
            offset: 2624,
        },
        /* J10 idx 108 */
        Magic {
            mask: 0x400200100080040000000000,
            magic: 0x008825012041802a1824110ca4204000,
            shift: 123,
            offset: 2656,
        },
        /* J11 idx 109 */
        Magic {
            mask: 0x800400200100080000000000,
            magic: 0x004412501021008a0c24910620460000,
            shift: 123,
            offset: 2688,
        },
        /* K1  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K2  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K3  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K4  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K5  off     */
        Magic {
            mask: 0,
            magic: 0,
            shift: 128,
            offset: 2720,
        },
        /* K6  idx 115 */
        Magic {
            mask: 0x040020010008004000000000,
            magic: 0x028c1145108510ca44845106208a0000,
            shift: 123,
            offset: 2720,
        },
        /* K7  idx 116 */
        Magic {
            mask: 0x080040020010008000000000,
            magic: 0x0108a30485304a922448490a20460000,
            shift: 123,
            offset: 2752,
        },
        /* K8  idx 117 */
        Magic {
            mask: 0x100080040020010000000000,
            magic: 0x0088e505244181aa1a249114ac308000,
            shift: 123,
            offset: 2784,
        },
        /* K9  idx 118 */
        Magic {
            mask: 0x200100080040020000000000,
            magic: 0x01082300853048922048110a20420000,
            shift: 123,
            offset: 2816,
        },
        /* K10 idx 119 */
        Magic {
            mask: 0x400200100080040000000000,
            magic: 0x008825012041802a1824110ca4204000,
            shift: 123,
            offset: 2848,
        },
        /* K11 idx 120 */
        Magic {
            mask: 0x800400200100080000000000,
            magic: 0x004412501021008a0c24910620460000,
            shift: 123,
            offset: 2880,
        },
    ];
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

    #[inline(always)]
    fn get_king_attacks(&self, start_idx: u8) -> u128 {
        Self::KING_ATTACKS[start_idx as usize]
    }

    #[inline(always)]
    fn get_knight_attacks(&self, start_idx: u8) -> u128 {
        Self::KNIGHT_ATTACKS[start_idx as usize]
    }

    fn get_bishop_attacks(&self, start_idx: u8, occ: u128) -> u128 {
        let m = Self::BISHOP_MAGICS[start_idx as usize];
        if m.mask == 0 {
            return 0;
        }

        let hash = (((occ & m.mask).wrapping_mul(m.magic)) >> m.shift) as usize;
        unsafe { BISHOP_ATTACKS_DB[m.offset + hash] }
    }

    fn get_rook_attacks(&self, start_idx: u8, occ: u128) -> u128 {
        let mr = Self::ROOK_R_MAGICS[start_idx as usize];
        let hash_r = if mr.mask != 0 {
            (((occ & mr.mask).wrapping_mul(mr.magic)) >> mr.shift) as usize
        } else {
            0
        };

        let mc = Self::ROOK_C_MAGICS[start_idx as usize];
        let hash_c = if mc.mask != 0 {
            (((occ & mc.mask).wrapping_mul(mc.magic)) >> mc.shift) as usize
        } else {
            0
        };

        let mx = Self::ROOK_X_MAGICS[start_idx as usize];
        let hash_x = if mx.mask != 0 {
            (((occ & mx.mask).wrapping_mul(mx.magic)) >> mx.shift) as usize
        } else {
            0
        };

        unsafe {
            ROOK_R_ATTACKS_DB[mr.offset + hash_r]
                | ROOK_C_ATTACKS_DB[mc.offset + hash_c]
                | ROOK_X_ATTACKS_DB[mx.offset + hash_x]
        }
    }

    fn get_opp_covered_squares(&self) -> u128 {
        let mut res = 0;
        let (side_mask, opp_mask) = if !self.is_white {
            (self.board.white_pieces(), self.board.black_pieces())
        } else {
            (self.board.black_pieces(), self.board.white_pieces())
        };
        let all_occ = side_mask | opp_mask;
        let empty_or_enemy = !side_mask;

        if self.is_white {
            let mut bishops = self.board.white_bishops();
            while bishops != 0 {
                let from_idx = bishops.trailing_zeros() as u8;

                let mut attack_bb = self.get_bishop_attacks(from_idx, all_occ) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }
                    attack_bb &= attack_bb - 1;
                }

                bishops &= bishops - 1;
            }

            let mut rooks = self.board.white_rooks();
            while rooks != 0 {
                let from_idx = rooks.trailing_zeros() as u8;

                let mut attack_bb = self.get_rook_attacks(from_idx, all_occ) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }

                    attack_bb &= attack_bb - 1;
                }

                rooks &= rooks - 1;
            }

            let mut queens = self.board.white_queens();
            while queens != 0 {
                let from_idx = queens.trailing_zeros() as u8;

                let mut attack_bb = (self.get_rook_attacks(from_idx, all_occ)
                    | self.get_bishop_attacks(from_idx, all_occ))
                    & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }

                    attack_bb &= attack_bb - 1;
                }

                queens &= queens - 1;
            }

            let mut knights = self.board.white_knights();
            while knights != 0 {
                let from_idx = knights.trailing_zeros() as u8;

                let mut attack_bb = self.get_knight_attacks(from_idx) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }

                    attack_bb &= attack_bb - 1;
                }

                knights &= knights - 1;
            }

            let king = self.board.white_kings();
            let from_idx = king.trailing_zeros() as u8;

            let mut attack_bb = self.get_king_attacks(from_idx) & empty_or_enemy;
            while attack_bb != 0 {
                let to_idx = attack_bb.trailing_zeros() as u8;
                let target_piece = self.board.get_piece(to_idx);

                if !target_piece.is_empty() {
                    res |= 1u128 << to_idx;
                }

                attack_bb &= attack_bb - 1;
            }

            for i in 0..121 {
                let piece = self.board.get_piece(i);
                if piece.piece_type() != PieceType::Pawn || !piece.is_white() {
                    continue;
                }

                let en_passant_bb = self
                    .en_passant_square
                    .map(|sq| 1 << u8::from(sq))
                    .unwrap_or(0);
                if ((1u128 << (i + 1)) & en_passant_bb) != 0 {
                    res |= 1u128 << (i + 1);
                }

                if ((1 << (i + 11)) & en_passant_bb) != 0 {
                    res |= 1u128 << (i + 11);
                }

                if 1u128 << (i + 1) != 0 {
                    res |= 1u128 << (i + 1);
                }

                if 1u128 << (i - 11) != 0 {
                    res |= 1u128 << (i + 11);
                }
            }
        } else {
            let mut bishops = self.board.black_bishops();
            while bishops != 0 {
                let from_idx = bishops.trailing_zeros() as u8;

                let mut attack_bb = self.get_bishop_attacks(from_idx, all_occ) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }

                    attack_bb &= attack_bb - 1;
                }

                bishops &= bishops - 1;
            }

            let mut rooks = self.board.black_rooks();
            while rooks != 0 {
                let from_idx = rooks.trailing_zeros() as u8;

                let mut attack_bb = self.get_rook_attacks(from_idx, all_occ) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }

                    attack_bb &= attack_bb - 1;
                }

                rooks &= rooks - 1;
            }

            let mut queens = self.board.black_queens();
            while queens != 0 {
                let from_idx = queens.trailing_zeros() as u8;

                let mut attack_bb = (self.get_rook_attacks(from_idx, all_occ)
                    | self.get_bishop_attacks(from_idx, all_occ))
                    & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }

                    attack_bb &= attack_bb - 1;
                }

                queens &= queens - 1;
            }

            let mut knights = self.board.black_knights();
            while knights != 0 {
                let from_idx = knights.trailing_zeros() as u8;

                let mut attack_bb = self.get_knight_attacks(from_idx) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    if !target_piece.is_empty() {
                        res |= 1u128 << to_idx;
                    }

                    attack_bb &= attack_bb - 1;
                }

                knights &= knights - 1;
            }

            let king = self.board.black_kings();
            let from_idx = king.trailing_zeros() as u8;

            let mut attack_bb = self.get_king_attacks(from_idx) & empty_or_enemy;
            while attack_bb != 0 {
                let to_idx = attack_bb.trailing_zeros() as u8;
                let target_piece = self.board.get_piece(to_idx);

                if !target_piece.is_empty() {
                    res |= 1u128 << to_idx;
                }

                attack_bb &= attack_bb - 1;
            }

            for i in 0..121 {
                let piece = self.board.get_piece(i);
                if piece.piece_type() != PieceType::Pawn || piece.is_white() {
                    continue;
                }
                let sq = i;

                let en_passant_bb = self
                    .en_passant_square
                    .map(|sq| 1 << u8::from(sq))
                    .unwrap_or(0);
                if ((1u128 << (i - 1)) & en_passant_bb) != 0 {
                    res |= 1u128 << (i - 1);
                }

                if ((1u128 << (i - 11)) & en_passant_bb) != 0 {
                    res |= 1u128 << (i - 11);
                }

                if 1u128 << (i - 1) != 0 {
                    res |= 1u128 << (i - 1);
                }

                if 1u128 << (i - 11) != 0 {
                    res |= 1u128 << (i - 11);
                }
            }
        }
        res
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);

        const WHITE_STARTING_PAWNS: u128 = 0b0000100000000001000000000010000000000100000011111000000000000000000000000000000000000000000000000000000000000000000000000000000000;
        const BLACK_STARTING_PAWNS: u128 = WHITE_STARTING_PAWNS.reverse_bits() << 7;
        let (side_mask, opp_mask) = if self.is_white {
            (self.board.white_pieces(), self.board.black_pieces())
        } else {
            (self.board.black_pieces(), self.board.white_pieces())
        };
        let all_occ = side_mask | opp_mask;
        let empty_or_enemy = !side_mask;
        if self.is_white {
            let mut bishop_pins = [0u128; 11];
            let mut rook_pins = [0u128; 11];
            let opp_covered_squares = self.get_opp_covered_squares();
            let in_check = opp_covered_squares & self.board.wk != 0;

            let mut opp_bishops = self.board.bb | self.board.bq;
            let mut bishop_idx = 0;
            while opp_bishops != 0 {
                let from_idx = opp_bishops.trailing_zeros() as u8;
                bishop_pins[bishop_idx] = self.get_bishop_attacks(from_idx, all_occ);
                bishop_idx += 1;
                opp_bishops &= opp_bishops - 1;
            }

            let mut opp_rooks = self.board.br | self.board.bq;
            let mut rook_idx = 0;
            while opp_rooks != 0 {
                let from_idx = opp_bishops.trailing_zeros() as u8;
                rook_pins[rook_idx] = self.get_rook_attacks(from_idx, all_occ);
                rook_idx += 1;
                opp_rooks &= opp_rooks - 1; 
            }

            if !in_check {
                let mut bishops = self.board.white_bishops();
                while bishops != 0 {
                    let from_idx = bishops.trailing_zeros() as u8;

                    let mut attack_bb = self.get_bishop_attacks(from_idx, all_occ) & empty_or_enemy;
                    while attack_bb != 0 {
                        let to_idx = attack_bb.trailing_zeros() as u8;
                        let target_piece = self.board.get_piece(to_idx);

                        moves.push(Move::new(
                            from_idx,
                            to_idx,
                            PieceType::None,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        attack_bb &= attack_bb - 1;
                    }

                    bishops &= bishops - 1;
                }

                let mut rooks = self.board.white_rooks();
                while rooks != 0 {
                    let from_idx = rooks.trailing_zeros() as u8;

                    let mut attack_bb = self.get_rook_attacks(from_idx, all_occ) & empty_or_enemy;
                    while attack_bb != 0 {
                        let to_idx = attack_bb.trailing_zeros() as u8;
                        let target_piece = self.board.get_piece(to_idx);

                        moves.push(Move::new(
                            from_idx,
                            to_idx,
                            PieceType::None,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        attack_bb &= attack_bb - 1;
                    }

                    rooks &= rooks - 1;
                }

                let mut queens = self.board.white_queens();
                while queens != 0 {
                    let from_idx = queens.trailing_zeros() as u8;

                    let mut attack_bb = (self.get_rook_attacks(from_idx, all_occ)
                        | self.get_bishop_attacks(from_idx, all_occ))
                        & empty_or_enemy;
                    while attack_bb != 0 {
                        let to_idx = attack_bb.trailing_zeros() as u8;
                        let target_piece = self.board.get_piece(to_idx);

                        moves.push(Move::new(
                            from_idx,
                            to_idx,
                            PieceType::None,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        attack_bb &= attack_bb - 1;
                    }

                    queens &= queens - 1;
                }

                let mut knights = self.board.white_knights();
                while knights != 0 {
                    let from_idx = knights.trailing_zeros() as u8;
                    let mut attack_bb = self.get_knight_attacks(from_idx) & empty_or_enemy;
                    while attack_bb != 0 {
                        let to_idx = attack_bb.trailing_zeros() as u8;
                        let target_piece = self.board.get_piece(to_idx);

                        moves.push(Move::new(
                            from_idx,
                            to_idx,
                            PieceType::None,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        attack_bb &= attack_bb - 1;
                    }

                    knights &= knights - 1;
                }

                

                for i in 0..121 {
                    let piece = self.board.get_piece(i);
                    if piece.piece_type() != PieceType::Pawn || !piece.is_white() {
                        continue;
                    }

                    let single_push = (1 << (i + 12)) & !all_occ;
                    if single_push != 0 {
                        if !matches!(i + 12, 116 | 117 | 118 | 119 | 120 | 109 | 98 | 87 | 76) {
                            moves.push(Move::new(
                                i,
                                i + 12,
                                PieceType::None,
                                PieceType::None,
                                false,
                                false,
                                false,
                            ));
                        } else {
                            moves.push(Move::new(
                                i,
                                i + 12,
                                PieceType::Queen,
                                PieceType::None,
                                false,
                                false,
                                true,
                            ));
                            moves.push(Move::new(
                                i,
                                i + 12,
                                PieceType::Knight,
                                PieceType::None,
                                false,
                                false,
                                true,
                            ));
                            moves.push(Move::new(
                                i,
                                i + 12,
                                PieceType::Rook,
                                PieceType::None,
                                false,
                                false,
                                true,
                            ));
                            moves.push(Move::new(
                                i,
                                i + 12,
                                PieceType::Knight,
                                PieceType::None,
                                false,
                                false,
                                true,
                            ));
                        }
                    }

                    if ((1 << i) & WHITE_STARTING_PAWNS) != 0 && single_push != 0 {
                        moves.push(Move::new(
                            i,
                            i + 24,
                            PieceType::None,
                            PieceType::None,
                            true,
                            false,
                            false,
                        ));
                    }

                    let en_passant_bb = self
                        .en_passant_square
                        .map(|sq| 1 << u8::from(sq))
                        .unwrap_or(0);
                    if ((1 << (i + 1)) & en_passant_bb) != 0 {
                        moves.push(Move::new(
                            i,
                            i + 1,
                            PieceType::None,
                            PieceType::None,
                            false,
                            true,
                            false,
                        ));
                    }

                    if ((1 << (i + 11)) & en_passant_bb) != 0 {
                        moves.push(Move::new(
                            i,
                            i + 11,
                            PieceType::None,
                            PieceType::None,
                            false,
                            true,
                            false,
                        ));
                    }

                    if ((1 << (i + 1)) & opp_mask) != 0 {
                        let target_piece = self.board.get_piece(i + 1);
                        if !matches!(i + 1, 116 | 117 | 118 | 119 | 120) {
                            moves.push(Move::new(
                                i,
                                i + 1,
                                PieceType::None,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                        } else {
                            let res_sq = i + 1;
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Queen,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Knight,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Rook,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Bishop,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                        }
                    }

                    if ((1 << (i + 11)) & opp_mask) != 0 {
                        let target_piece = self.board.get_piece(i + 11);
                        if !matches!(i + 11, 120 | 109 | 98 | 87 | 76) {
                            moves.push(Move::new(
                                i,
                                i + 11,
                                PieceType::None,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                        } else {
                            let res_sq = i + 11;
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Queen,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Knight,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Rook,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                            moves.push(Move::new(
                                i,
                                res_sq,
                                PieceType::Bishop,
                                target_piece.piece_type(),
                                false,
                                false,
                                false,
                            ));
                        }
                    }
                }
            } else {
                let mut rook_attack_idx = None;
                for (i, attack) in rook_pins.iter().enumerate() {
                    if attack & self.board.wk != 0 {
                        rook_attack_idx = Some(i);
                    }
                }

                let mut bishop_attack_idx = None;
                for (i, attack) in rook_pins.iter().enumerate() {
                    if attack & self.board.wk != 0 {
                        bishop_attack_idx = Some(i);
                    }
                }

                if !rook_attack_idx.is_some() && !bishop_attack_idx.is_some() {
                    //
                }
            }

            let king = self.board.white_kings();
            let from_idx = king.trailing_zeros() as u8;

            let mut attack_bb = self.get_king_attacks(from_idx) & empty_or_enemy & opp_covered_squares;
            while attack_bb != 0 {
                let to_idx = attack_bb.trailing_zeros() as u8;
                let target_piece = self.board.get_piece(to_idx);

                moves.push(Move::new(
                    from_idx,
                    to_idx,
                    PieceType::None,
                    target_piece.piece_type(),
                    false,
                    false,
                    false,
                ));
                attack_bb &= attack_bb - 1;
            }
        } else {
            let mut bishops = self.board.black_bishops();
            while bishops != 0 {
                let from_idx = bishops.trailing_zeros() as u8;

                let mut attack_bb = self.get_bishop_attacks(from_idx, all_occ) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    moves.push(Move::new(
                        from_idx,
                        to_idx,
                        PieceType::None,
                        target_piece.piece_type(),
                        false,
                        false,
                        false,
                    ));
                    attack_bb &= attack_bb - 1;
                }

                bishops &= bishops - 1;
            }

            let mut rooks = self.board.black_rooks();
            while rooks != 0 {
                let from_idx = rooks.trailing_zeros() as u8;

                let mut attack_bb = self.get_rook_attacks(from_idx, all_occ) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);
                    moves.push(Move::new(
                        from_idx,
                        to_idx,
                        PieceType::None,
                        target_piece.piece_type(),
                        false,
                        false,
                        false,
                    ));
                    attack_bb &= attack_bb - 1;
                }

                rooks &= rooks - 1;
            }

            let mut queens = self.board.black_queens();
            while queens != 0 {
                let from_idx = queens.trailing_zeros() as u8;

                let mut attack_bb = (self.get_rook_attacks(from_idx, all_occ)
                    | self.get_bishop_attacks(from_idx, all_occ))
                    & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    moves.push(Move::new(
                        from_idx,
                        to_idx,
                        PieceType::None,
                        target_piece.piece_type(),
                        false,
                        false,
                        false,
                    ));
                    attack_bb &= attack_bb - 1;
                }

                queens &= queens - 1;
            }

            let mut knights = self.board.black_knights();
            while knights != 0 {
                let from_idx = knights.trailing_zeros() as u8;

                let mut attack_bb = self.get_knight_attacks(from_idx) & empty_or_enemy;
                while attack_bb != 0 {
                    let to_idx = attack_bb.trailing_zeros() as u8;
                    let target_piece = self.board.get_piece(to_idx);

                    moves.push(Move::new(
                        from_idx,
                        to_idx,
                        PieceType::None,
                        target_piece.piece_type(),
                        false,
                        false,
                        false,
                    ));
                    attack_bb &= attack_bb - 1;
                }

                knights &= knights - 1;
            }

            let king = self.board.black_kings();
            let from_idx = king.trailing_zeros() as u8;

            let mut attack_bb =
                self.get_king_attacks(from_idx) & empty_or_enemy & self.get_opp_covered_squares();
            while attack_bb != 0 {
                let to_idx = attack_bb.trailing_zeros() as u8;
                let target_piece = self.board.get_piece(to_idx);

                moves.push(Move::new(
                    from_idx,
                    to_idx,
                    PieceType::None,
                    target_piece.piece_type(),
                    false,
                    false,
                    false,
                ));
                attack_bb &= attack_bb - 1;
            }

            for i in 0..121 {
                let piece = self.board.get_piece(i);
                if piece.piece_type() != PieceType::Pawn || piece.is_white() {
                    continue;
                }
                let sq = i;

                let single_push = (1u128 << (i - 12)) & !all_occ;
                if single_push != 0 {
                    if !matches!(i - 12, 4 | 3 | 2 | 1 | 0 | 11 | 22 | 33 | 44) {
                        moves.push(Move::new(
                            sq,
                            i - 12,
                            PieceType::None,
                            PieceType::None,
                            false,
                            false,
                            false,
                        ));
                    } else {
                        let res_sq = i - 12;
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Queen,
                            PieceType::None,
                            false,
                            false,
                            true,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Knight,
                            PieceType::None,
                            false,
                            false,
                            true,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Rook,
                            PieceType::None,
                            false,
                            false,
                            true,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Knight,
                            PieceType::None,
                            false,
                            false,
                            true,
                        ));
                    }
                }

                if ((1u128 << i) & BLACK_STARTING_PAWNS) != 0 && single_push != 0 {
                    moves.push(Move::new(
                        sq,
                        i - 24,
                        PieceType::None,
                        PieceType::None,
                        true,
                        false,
                        false,
                    ));
                }

                let en_passant_bb = self
                    .en_passant_square
                    .map(|sq| 1 << u8::from(sq))
                    .unwrap_or(0);
                if ((1u128 << (i - 1)) & en_passant_bb) != 0 {
                    moves.push(Move::new(
                        sq,
                        i - 1,
                        PieceType::None,
                        PieceType::None,
                        false,
                        true,
                        false,
                    ));
                }

                if ((1u128 << (i - 11)) & en_passant_bb) != 0 {
                    moves.push(Move::new(
                        sq,
                        i - 11,
                        PieceType::None,
                        PieceType::None,
                        false,
                        true,
                        false,
                    ));
                }

                if ((1u128 << (i - 1)) & opp_mask) != 0 {
                    let target_piece = self.board.get_piece(i - 1);
                    if !matches!(i - 1, 4 | 3 | 2 | 1) {
                        moves.push(Move::new(
                            sq,
                            i - 1,
                            PieceType::None,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                    } else {
                        let res_sq = i - 1;
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Queen,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Knight,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Rook,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Bishop,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                    }
                }

                if ((1u128 << (i - 11)) & opp_mask) != 0 {
                    let target_piece = self.board.get_piece(i - 11);
                    if !matches!(i - 11, 11 | 22 | 33 | 44 | 55) {
                        moves.push(Move::new(
                            sq,
                            i - 11,
                            PieceType::None,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                    } else {
                        let res_sq = i - 11;
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Queen,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Knight,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Rook,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                        moves.push(Move::new(
                            sq,
                            res_sq,
                            PieceType::Bishop,
                            target_piece.piece_type(),
                            false,
                            false,
                            false,
                        ));
                    }
                }
            }
        }
        moves
    }
}
