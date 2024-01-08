use crate::position;
use crate::moves;

const KNIGHT_VALUE: isize = 3;
const BISHOP_VALUE: isize = 3;
const ROOK_VALUE: isize = 5;
const QUEEN_VALUE: isize = 9;

const KING_MOBILITY_BONUS: isize = 50;

const PAWN_MASK: u64 = u64::from_be_bytes([
    0b00000000,
    0b00000000,
    0b00011000,
    0b00111100,
    0b01111110,
    0b11100111,
    0b11100111,
    0b00000000,
]);

const KNIGHT_MASK: u64 = u64::from_be_bytes([
    0b00000000,
    0b00000000,
    0b01111110,
    0b01111110,
    0b01111110,
    0b01111110,
    0b00000000,
    0b00000000,
]);

const BISHOP_MASK: u64 = u64::from_be_bytes([
    0b00000000,
    0b00000000,
    0b00000000,
    0b11111111,
    0b11100111,
    0b11100111,
    0b11000011,
    0b00000000,
]);

const ROOK_MASK: u64 = u64::from_be_bytes([
    0b11111111,
    0b11111111,
    0b11000011,
    0b11000011,
    0b11000011,
    0b11000011,
    0b11111111,
    0b11111111,
]);

const QUEEN_MASK: u64 = u64::from_be_bytes([
    0b11111111,
    0b11111111,
    0b11100111,
    0b11000011,
    0b11000011,
    0b11100111,
    0b11111111,
    0b11111111,
]);

const KING_MASK: u64 = u64::from_be_bytes([
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b00000000,
    0b11111111,
    0b11111111,
]);

const fn byte_swap(m: u64) -> u64 { u64::from_be_bytes(m.to_le_bytes()) }

pub fn evaluate(pos: &position::Position) -> isize {
    if pos.player(pos.turn).kings.size() != 1  { return isize::MIN }
    if pos.player(!pos.turn).kings.size() != 1 { return isize::MAX }

    let sign = match pos.turn { 
        position::Color::Black => -1,
        position::Color::White =>  1,
    };

    let material_evaluation =
        (pos.white.pawns.size()   as isize - pos.black.pawns.size()   as isize)                +
        (pos.white.knights.size() as isize - pos.black.knights.size() as isize) * KNIGHT_VALUE +
        (pos.white.bishops.size() as isize - pos.black.bishops.size() as isize) * BISHOP_VALUE +
        (pos.white.rooks.size()   as isize - pos.black.rooks.size()   as isize) * ROOK_VALUE   +
        (pos.white.queens.size()  as isize - pos.black.queens.size()  as isize) * QUEEN_VALUE;

    let virt = pos.to_virtual_position();
    let virt_evaluation =
        (virt.white.pawns.size()   as isize - virt.black.pawns.size()   as isize)                +
        (virt.white.knights.size() as isize - virt.black.knights.size() as isize) * KNIGHT_VALUE +
        (virt.white.bishops.size() as isize - virt.black.bishops.size() as isize) * BISHOP_VALUE +
        (virt.white.rooks.size()   as isize - virt.black.rooks.size()   as isize) * ROOK_VALUE   +
        (virt.white.queens.size()  as isize - virt.black.queens.size()  as isize) * QUEEN_VALUE;

    let mut piece_happiness = 0;
    for (white_board, black_board, mask, weight) in [
        (virt.white.pawns.0,   virt.black.pawns.0,   PAWN_MASK,   1),
        (virt.white.knights.0, virt.black.knights.0, KNIGHT_MASK, 2),
        (virt.white.bishops.0, virt.black.bishops.0, BISHOP_MASK, 2),
        (virt.white.rooks.0,   virt.black.rooks.0,   ROOK_MASK,   5),
        (virt.white.queens.0,  virt.black.queens.0,  QUEEN_MASK,  5),
        (pos.white.kings.0,    pos.black.kings.0,    KING_MASK,   20),
    ] {
        piece_happiness += weight 
            * ( (white_board & mask).count_ones() as isize - (black_board & byte_swap(mask)).count_ones() as isize);
    }

    let mobility_score = |piece|  match piece {
        position::Piece::Queen  => 10,
        position::Piece::Rook   => 8,
        position::Piece::Bishop => 6,
        position::Piece::King   => 5,
        position::Piece::Knight => 4,
        position::Piece::Pawn   => -10,
    };

    piece_happiness += 
        mobility_score(pos.white.piece_at(pos.white.kings.into_iter().next().unwrap()).unwrap()) -
        mobility_score(pos.black.piece_at(pos.black.kings.into_iter().next().unwrap()).unwrap());


    sign * (material_evaluation * 10000 + virt_evaluation * 1000 + piece_happiness * 100)
}
