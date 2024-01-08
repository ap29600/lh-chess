use crate::position::*;
use crate::moves;
use std::io::prelude::*;

pub struct HumanPlayer {
    game_history: Vec<Position>,
}

impl HumanPlayer {
    pub fn from_position(pos: &Position) -> Self {
        Self{ game_history: vec![pos.clone()] }
    }
    fn current_position(&self) -> &Position {
        self.game_history.last().expect("a valid game has a position")
    }
}

impl crate::Player for HumanPlayer {
    fn suggest_move(&mut self) -> moves::Move {

        let mut line = String::new();
        let pos = self.current_position();

        let promotion_rank = match pos.turn {
            Color::White => 7,
            Color::Black => 0,
        };

        let valid_moves = moves::moves(pos);

        loop {
            println!("{}", pos);
            print!("> ");
            std::io::stdout().flush().expect("can write to stdout");

            line.clear();
            std::io::stdin().read_line(&mut line).expect("can read input");
            let (from, to, promote_to) = match *line.trim().split(' ').collect::<Vec<_>>() {
                [from, to, promote_to] => (from, to, Some(promote_to)),
                [from, to] => (from, to, None),
                _ => {
                    println!("Invalid format (should be '<start-square> <end-square>')");
                    continue
                },
            };

            let from = match from.as_bytes() {
                [file @ b'A'..=b'H', rank @ b'1'..=b'8'] => Square::new(rank - b'1', file - b'A'),
                _ => {
                    println!("start square is malformed (should be [A-H][1-8])");
                    continue
                },
            };

            let to = match to.as_bytes() {
                [file @ b'A'..=b'H', rank @ b'1'..=b'8'] => Square::new(rank - b'1', file - b'A'),
                _ => {
                    println!("end square is malformed (should be [A-H][1-8])");
                    continue
                },
            };

            let Some(moved_piece) = pos.player(pos.turn).piece_at(from) else { continue };
            let must_promote = moved_piece == Piece::Pawn && to.rank() == promotion_rank;

            let promote_to = match (must_promote, promote_to) {
                (true, Some(piece)) => Some(match piece {
                    "n" | "N" => Piece::Knight,
                    "b" | "B" => Piece::Bishop,
                    "r" | "R" => Piece::Rook,
                    "q" | "Q" => Piece::Queen,
                    _ => {
                        println!("Promotion piece must be one of [nNbBrRqQ]");
                        continue
                    }
                }),
                (false, None) => None,
                (true, None) => {
                    println!("Specify a piece to promote to");
                    continue 
                },
                (false, Some(_)) => {
                    println!("You can't promote with that move");
                    continue
                }
            };

            let captured_piece = pos.player(!pos.turn).piece_at(to);

            let mov = moves::Move {
                moved_piece,
                from,
                to,
                captured_piece,
                promote_to,
            };

            if valid_moves.contains(&mov) {
                for response in moves::moves(&moves::apply_move(&pos, mov)) {
                    if response.captured_piece == Some(Piece::King) {
                        println!("That move leaves the king hanging! ({})", response);
                        continue;
                    }
                }
                return mov;
            } else {
                println!("That move is not valid in this position");
                continue
            }
        }
    }

    fn input_move(&mut self, mov: moves::Move) {
        self.game_history.push(moves::apply_move(self.current_position(), mov))
    }
}
