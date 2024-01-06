#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.write_str(match self {
                Piece::Pawn   => "p",
                Piece::Knight => "n",
                Piece::Bishop => "b",
                Piece::Rook   => "r",
                Piece::Queen  => "q",
                Piece::King   => "k",
            })
        } else {
            f.write_str(match self {
                Piece::Pawn   => "P",
                Piece::Knight => "N",
                Piece::Bishop => "B",
                Piece::Rook   => "R",
                Piece::Queen  => "Q",
                Piece::King   => "K",
            })
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum Color {
    #[default] White,
    Black,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Color::White => "White",
            Color::Black => "Black",
        })
    }
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Square { 
    file: u8,
    rank: u8,
}

impl Square {
    pub fn new(rank: u8, file: u8) -> Self {
        assert!(rank < 8);
        assert!(file < 8);
        Self{rank, file}
    }

    pub fn rank(&self) -> u8 {
        self.rank
    }

    // pub fn file(&self) -> u8 {
    //     self.file
    // }

    pub fn to_bit_position(&self) -> u8 {
        let Self{file, rank} = self;
        rank * 8 + file
    }

    pub fn from_bit_position(position: u8) -> Self {
        Self{file: position % 8, rank: position / 8}
    }
}

impl std::ops::Add<SquareDiff> for Square {
    type Output = Option<Square>;
    fn add(self, diff: SquareDiff) -> Self::Output {
        let rank = self.rank as i8 + diff.rank_diff;
        let file = self.file as i8 + diff.file_diff;

        if file < 0 || 8 <= file { return None; }
        if rank < 0 || 8 <= rank { return None; }

        Some(Self{file: file as u8, rank: rank as u8})
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}{}", (b'A' + self.file) as char, (b'1' + self.rank) as char))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SquareDiff {
    pub file_diff: i8,
    pub rank_diff: i8,
}

impl std::ops::Mul<isize> for SquareDiff {
    type Output = SquareDiff;

    fn mul(self, rhs: isize) -> Self::Output {
        Self{file_diff: self.file_diff * rhs as i8, rank_diff: self.rank_diff * rhs as i8}
    }
}

impl SquareDiff {
    pub const fn to_bit_offset_and_mask(&self) -> (i8, u64) {

        const fn bit_range(start: u8, end: u8) -> u64 {
            let a = if end == 63 { u64::MAX } else { (1 << end) - 1 };
            let b = if start == 63 { u64::MAX } else { (1 << start) - 1 };
            return a &! b
        }

        if self.rank_diff.abs() >= 8 || self.file_diff.abs() >= 8 { return (0, 0); }

        let offset = self.rank_diff * 8 + self.file_diff;

        let row_mask = if self.rank_diff >= 0 {
            u64::MAX >> (self.rank_diff * 8)
        } else {
            u64::MAX << (-self.rank_diff * 8)
        };

        let file_mask = if self.file_diff >= 0 { 
            bit_range(0, 8 - self.file_diff as u8) 
        } else {
            bit_range((-self.file_diff) as u8, 8) 
        };

        let file_mask = 0
            | file_mask << 56 
            | file_mask << 48 
            | file_mask << 40 
            | file_mask << 32 
            | file_mask << 24 
            | file_mask << 16
            | file_mask << 8 
            | file_mask << 0 ;

        (offset, row_mask & file_mask)
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Default)]
pub struct BitBoard ( pub u64 );
impl BitBoard {
    pub fn size(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn has(&self, square: Square) -> bool {
        (self.0 >> square.to_bit_position()) & 1 > 0
    }

    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << square.to_bit_position();
    }

    pub fn unset(&mut self, square: Square) {
        self.0 &= !(1 << square.to_bit_position());
    }
}

impl std::fmt::Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() { 
            for file in 0..8 {
                let square = Square{rank, file};
                f.write_str(" ")?;
                if self.has(square) {
                    f.write_str("#")?;
                } else {
                    f.write_str(".")?;
                }
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Default, Clone)]
pub struct PlayerPieces {
    pub pawns:   BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
    pub rooks:   BitBoard,
    pub queens:  BitBoard,
    pub kings:   BitBoard,
}

impl PlayerPieces {
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        if self.pawns.has(square)   { return Some(Piece::Pawn);   }
        if self.knights.has(square) { return Some(Piece::Knight); }
        if self.bishops.has(square) { return Some(Piece::Bishop); }
        if self.rooks.has(square)   { return Some(Piece::Rook);   }
        if self.queens.has(square)  { return Some(Piece::Queen);  }
        if self.kings.has(square)   { return Some(Piece::King);   }
        None
    }

    // pub fn pieces(&self, kind: Piece) -> &BitBoard {
    //     match kind {
    //         Piece::Pawn => &self.pawns,
    //         Piece::Knight => &self.knights,
    //         Piece::Bishop => &self.bishops,
    //         Piece::Rook => &self.rooks,
    //         Piece::Queen => &self.queens,
    //         Piece::King => &self.kings,
    //     }
    // }

    pub fn mut_pieces(&mut self, kind: Piece) -> &mut BitBoard {
        match kind {
            Piece::Pawn => &mut self.pawns,
            Piece::Knight => &mut self.knights,
            Piece::Bishop => &mut self.bishops,
            Piece::Rook => &mut self.rooks,
            Piece::Queen => &mut self.queens,
            Piece::King => &mut self.kings,
        }
    }

    pub fn all(&self) -> BitBoard {
        BitBoard(0
            | self.pawns.0
            | self.knights.0
            | self.bishops.0
            | self.rooks.0
            | self.queens.0
            | self.kings.0)
    }
}

#[derive(Eq, PartialEq, Hash, Default, Clone)]
pub struct Position {
    pub white: PlayerPieces,
    pub black: PlayerPieces,
    pub turn: Color,
}

impl TryFrom<&str> for Position {
    type Error = Square;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut result = Self::default();
        let mut rank = 7;
        let mut file = 0;
        for c in s.chars() {
            if c == '\n' {
                if rank == 0 { break }
                rank -= 1;
                file  = 0;
                continue 
            }
            if file == 8 {
                return Err(Square{rank, file:file - 1});
            }
            let square = Square::new(rank, file);
            match c {
                'p' => { result.black.pawns.0   |= 1 << square.to_bit_position(); },
                'n' => { result.black.knights.0 |= 1 << square.to_bit_position(); },
                'b' => { result.black.bishops.0 |= 1 << square.to_bit_position(); },
                'r' => { result.black.rooks.0   |= 1 << square.to_bit_position(); },
                'q' => { result.black.queens.0  |= 1 << square.to_bit_position(); },
                'k' => { result.black.kings.0   |= 1 << square.to_bit_position(); },

                'P' => { result.white.pawns.0   |= 1 << square.to_bit_position(); },
                'N' => { result.white.knights.0 |= 1 << square.to_bit_position(); },
                'B' => { result.white.bishops.0 |= 1 << square.to_bit_position(); },
                'R' => { result.white.rooks.0   |= 1 << square.to_bit_position(); },
                'Q' => { result.white.queens.0  |= 1 << square.to_bit_position(); },
                'K' => { result.white.kings.0   |= 1 << square.to_bit_position(); },
                '.' => (),
                _   => { return Err(Square{rank, file}) },
            }
            file += 1;
        }
        Ok(result)
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = [[" ."; 8];8];
        for (bit_board, c) in [
            (self.black.pawns,   " ♙"),
            (self.black.knights, " ♘"),
            (self.black.bishops, " ♗"),
            (self.black.rooks,   " ♖"),
            (self.black.queens,  " ♕"),
            (self.black.kings,   " ♔"),
            (self.white.pawns,   " ♟︎"),
            (self.white.knights, " ♞"),
            (self.white.bishops, " ♝"),
            (self.white.rooks,   " ♜"),
            (self.white.queens,  " ♛"),
            (self.white.kings,   " ♚"),
        ] {
            for Square{file, rank} in bit_board.into_iter() {
                board[rank as usize][file as usize] = c;
            }
        }

        f.write_fmt(format_args!("{} to move:\n", self.turn))?;
        for (i, line) in board.iter().enumerate().rev() {
            f.write_fmt(format_args!("{} ", (b'1' + i as u8) as char))?;
            for square in line {
                f.write_str(*square)?;
            }
            f.write_fmt(format_args!("{}", '\n'))?;
        }
        f.write_str("\n   A B C D E F G H\n")?;
        Ok(())
    }
}

pub struct BitBoardIterator {
    board:    BitBoard,
    position: u8,
}


impl Iterator for BitBoardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= 64 { return None }
        let gap = (self.board.0 >> self.position).trailing_zeros() as u8;
        if gap + self.position >= 64 {
            None
        } else {
            let result = Some(Square::from_bit_position(gap + self.position));
            self.position += gap + 1;
            result
        }
    }
}

impl IntoIterator for BitBoard {
    type Item = Square;
    type IntoIter = BitBoardIterator;
    fn into_iter(self) -> Self::IntoIter {
        BitBoardIterator{ board: self, position: 0 }
    }
}

impl FromIterator<Square> for BitBoard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        let mut result = Self(0);
        for cell in iter {
            result.0 |= 1 << cell.to_bit_position();
        }
        result
    }
}

impl Position {
    pub fn all(&self) -> BitBoard {
        BitBoard(self.black.all().0 | self.white.all().0)
    }

    // pub fn piece_at(&self, square: Square) -> Option<(Color, Piece)> {
    //     self.white
    //         .piece_at(square)
    //         .map(|p| (Color::White, p))
    //         .or(self.black
    //             .piece_at(square)
    //             .map(|p| (Color::Black, p)))
    // }

    pub fn player(&self, color: Color) -> &PlayerPieces {
        match color {
            Color::Black => &self.black,
            Color::White => &self.white,
        }
    }

    pub fn mut_player(&mut self, color: Color) -> &mut PlayerPieces {
        match color {
            Color::Black => &mut self.black,
            Color::White => &mut self.white,
        }
    }

    pub fn to_virtual_position(&self) -> Position {
        fn rot_right(s: i8, b: u64) -> u64 {
            let (shift_l, mask_l) = SquareDiff{rank_diff: 0, file_diff: (s + 8) % 8}.to_bit_offset_and_mask();
            let (shift_r, mask_r) = SquareDiff{rank_diff: 0, file_diff: (s + 8) % 8 - 8}.to_bit_offset_and_mask();

            let result = (b & mask_l).overflowing_shl(shift_l.abs() as u32).0 | (b & mask_r).overflowing_shr(shift_r.abs() as u32).0;

            result
        }        

        let BitBoard(white_pieces) = self.white.all();
        let BitBoard(black_pieces) = self.black.all();
        let all_pieces = white_pieces | black_pieces;

        let mut result = Position::default();

        for (mut pieces, board) in [
            (self.white.pawns.0   | self.black.pawns.0,   &mut result.white.pawns.0),
            (self.white.knights.0 | self.black.knights.0, &mut result.white.knights.0),
            (self.white.bishops.0 | self.black.bishops.0, &mut result.white.bishops.0),
            (self.white.rooks.0   | self.black.rooks.0,   &mut result.white.rooks.0),
            (self.white.queens.0  | self.black.queens.0,  &mut result.white.queens.0),
            (self.white.kings.0   | self.black.kings.0,   &mut result.white.kings.0),
        ] {
            for _ in 0..8 {
                pieces = rot_right(1, pieces);
                *board |= pieces & white_pieces;
                pieces &= !all_pieces;
            }
        }

        for (mut pieces, board) in [
            (self.white.pawns.0   | self.black.pawns.0,   &mut result.black.pawns.0),
            (self.white.knights.0 | self.black.knights.0, &mut result.black.knights.0),
            (self.white.bishops.0 | self.black.bishops.0, &mut result.black.bishops.0),
            (self.white.rooks.0   | self.black.rooks.0,   &mut result.black.rooks.0),
            (self.white.queens.0  | self.black.queens.0,  &mut result.black.queens.0),
            (self.white.kings.0   | self.black.kings.0,   &mut result.black.kings.0),
        ] {
            for _ in 0..8 {
                pieces = rot_right(-1, pieces);
                *board |= pieces & black_pieces;
                pieces &= !all_pieces;
            }
        }

        result
    }
}


impl FromIterator<(Square, Color, Piece)> for Position {
    fn from_iter<T: IntoIterator<Item = (Square, Color, Piece)>>(iter: T) -> Self {
        let mut result = Self::default();
        for (square, color, piece) in iter {
            let bit_board = match (color, piece) {
                (Color::Black, Piece::Pawn)   => &mut result.black.pawns,
                (Color::Black, Piece::Knight) => &mut result.black.knights,
                (Color::Black, Piece::Bishop) => &mut result.black.bishops,
                (Color::Black, Piece::Rook)   => &mut result.black.rooks,
                (Color::Black, Piece::Queen)  => &mut result.black.queens,
                (Color::Black, Piece::King)   => &mut result.black.kings,
                (Color::White, Piece::Pawn)   => &mut result.white.pawns,
                (Color::White, Piece::Knight) => &mut result.white.knights,
                (Color::White, Piece::Bishop) => &mut result.white.bishops,
                (Color::White, Piece::Rook)   => &mut result.white.rooks,
                (Color::White, Piece::Queen)  => &mut result.white.queens,
                (Color::White, Piece::King)   => &mut result.white.kings,
            };
            bit_board.0 |= 1 << square.to_bit_position();
        }
        result
    }
}
