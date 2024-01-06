use crate::position::{Square, SquareDiff, Position, Color, BitBoard, Piece};
use crate::evaluation;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct Move {
    pub moved_piece: Piece, 
    pub from: Square, 
    pub to: Square,
    pub captured_piece: Option<Piece>,
    pub promote_to: Option<Piece>
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            f.write_fmt(format_args!("{:#}", self.moved_piece))?
        } else {
            f.write_fmt(format_args!("{}", self.moved_piece))?
        }

        f.write_fmt(format_args!("{}", self.from))?;

        if let Some(piece) = self.captured_piece {
            if f.alternate() {
                f.write_fmt(format_args!("x{}", piece))?
            } else {
                f.write_fmt(format_args!("x{:#}", piece))?
            }
        }
        f.write_fmt(format_args!("{}", self.to))?;

        if let Some(piece) = self.promote_to {
            if f.alternate() {
                f.write_fmt(format_args!("{:#}", piece))?
            } else {
                f.write_fmt(format_args!("{}", piece))?
            }
        }

        Ok(())
    }
}

pub fn moves(pos: &Position) -> Vec<Move> {

    let virt = pos.to_virtual_position();

    let own_forward = match pos.turn {
        Color::White =>  1,
        Color::Black => -1,
    };

    let own_virtual_pieces = match pos.turn {
        Color::Black => virt.black,
        Color::White => virt.white,
    };

    let own_promotion_rank = match pos.turn {
        Color::White => 7,
        Color::Black => 0,
    };

    let mut result = vec![];
    let mut emit_move = |from, to| {
        let moved_piece = pos.player(pos.turn).piece_at(from).expect("has moving piece");
        let captured_piece = pos.player(!pos.turn).piece_at(to);

        if to.rank() == own_promotion_rank && moved_piece == Piece::Pawn {
            for promote_to in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                result.push(Move{ from, to, moved_piece, captured_piece, promote_to: Some(promote_to) });
            }
        } else {
            result.push(Move{ from, to, moved_piece, captured_piece, promote_to: None });
        }
    };

    for start_square in own_virtual_pieces.pawns {

        // forward move
        if let Some(landing_square) = (start_square + SquareDiff{rank_diff: own_forward, file_diff: 0}) {
            if !pos.all().has(landing_square) { 
                emit_move(start_square, landing_square);
            }
        }

        for (rank_diff, file_diff) in [
            (own_forward, -1), (own_forward,  1),
        ] {
            let Some(landing_square) = (start_square + SquareDiff{rank_diff, file_diff}) else { continue };
            if pos.player(pos.turn).all().has(landing_square) { continue }   // self-capture is not allowed.
            if !pos.player(!pos.turn).all().has(landing_square) { continue } // diagonal move must be a capture.
            emit_move(start_square, landing_square);
        }
    }

    for start_square in own_virtual_pieces.knights {
        for (rank_diff, file_diff) in [
            /*    */  (-2, -1), /*    */  (-2,  1), /*    */
            (-1, -2), /*    */  /*    */  /*    */  (-1,  2),
            /*    */  /*    */  /*    */  /*    */  /*    */ 
            ( 1, -2), /*    */  /*    */  /*    */  ( 1,  2),
            /*    */  ( 2, -1), /*    */  ( 2,  1), /*    */
        ] {
            let Some(landing_square) = (start_square + SquareDiff{rank_diff, file_diff}) else { continue };
            if pos.player(pos.turn).all().has(landing_square) { continue } // self-capture is not allowed.
            emit_move(start_square, landing_square);
        }
    }

    for start_square in BitBoard(own_virtual_pieces.bishops.0 | own_virtual_pieces.queens.0) {
        for (rank_diff, file_diff) in [
            (-1, -1), /*    */  (-1,  1),
            /*    */  /*    */  /*    */
            ( 1, -1), /*    */  ( 1,  1),
        ] {
            let direction = SquareDiff{rank_diff, file_diff};
            for step in 1..7 {
                let Some(landing_square) = start_square + direction * step else { break };
                if pos.player(pos.turn).all().has(landing_square) { break }
                emit_move(start_square, landing_square);
                if pos.player(!pos.turn).all().has(landing_square) { break }
            }
        }
    }

    for start_square in BitBoard(own_virtual_pieces.rooks.0 | own_virtual_pieces.queens.0) {
        for (rank_diff, file_diff) in [
            /*    */  (-1,  0), /*    */ 
            ( 0, -1), /*    */  ( 0,  1),
            /*    */  ( 1,  0), /*    */ 
        ] {
            let direction = SquareDiff{rank_diff, file_diff};
            for step in 1..7 {
                let Some(landing_square) = start_square + direction * step else { break };
                if pos.player(pos.turn).all().has(landing_square) { break }
                emit_move(start_square, landing_square);
                if pos.player(!pos.turn).all().has(landing_square) { break }
            }
        }
    }

    for start_square in own_virtual_pieces.kings {
        for (rank_diff, file_diff) in [
            (-1, -1), (-1,  0), (-1, 1),
            ( 0, -1), /*    */  ( 0, 1),
            ( 1, -1), ( 1,  0), ( 1, 1),
        ] {
            let Some(landing_square) = (start_square + SquareDiff{rank_diff, file_diff}) else { continue };
            if pos.player(pos.turn).all().has(landing_square) { continue }
            emit_move(start_square, landing_square);
        }
    }

    result
}


pub fn apply_move(pos: &Position, mov: Move) -> Position {
    let mut result = pos.clone();
    result.turn = !result.turn;

    result.mut_player(pos.turn).mut_pieces(mov.moved_piece).unset(mov.from);

    if let Some(piece) = mov.captured_piece {
        result.mut_player(!pos.turn).mut_pieces(piece).unset(mov.to);
    }

    if let Some(piece) = mov.promote_to {
        result.mut_player(pos.turn).mut_pieces(piece).set(mov.to);
    } else {
        result.mut_player(pos.turn).mut_pieces(mov.moved_piece).set(mov.to);
    }

    result
}

pub struct Engine {
    game_history: Vec<Position>,
    recursion_depth_soft_cap: isize,
    recursion_depth_hard_cap: isize,
    // evaluation_cache: HashMap<Position, (isize, isize)>,
}


impl Engine {
    pub fn new() -> Self {
        Engine{
            game_history: vec![Position::try_from(
"rnbqkbnr
pppppppp
........
........
........
........
PPPPPPPP
RNBQKBNR").expect("hardcoded position is valid")],
            recursion_depth_soft_cap: 5,
            recursion_depth_hard_cap: 8,
            // evaluation_cache: HashMap::new(),
        }
    }

    pub fn suggest_and_play_move(&mut self) -> Option<(Move, isize)> {
        match self.suggest_move() {
            res @ Some((mov, _)) => { self.input_move(mov); res},
            res => res
        }
    }

    pub fn roll_back(&mut self) {
        if self.game_history.len() > 1 {
            self.game_history.pop();
        }
    }

    pub fn input_move(&mut self, mov: Move) {
        self.game_history.push(apply_move(self.current_position(), mov));
    }

    pub fn current_position(&self) -> &Position {
        self.game_history.last().expect("a valid engine always has some state")
    }

    const EVAL_MAX : isize = isize::MAX / 2;
    const EVAL_MIN : isize = isize::MIN / 2;

    pub fn suggest_move(&mut self) -> Option<(Move, isize)> {
        // if self.evaluation_cache.len() > 1_000_000_000 {
        //     self.evaluation_cache.clear()
        // }
        self.suggest_move_internal(0, Self::EVAL_MAX)
    }

    fn suggest_move_internal(&mut self, current_depth: isize, prune_threshold: isize) -> Option<(Move, isize)> {
        // stalemate condition
        if self.game_history.iter().filter(|&pos| pos == self.current_position()).count() > 2 {
            return None;
        }

        let current_position = self.current_position().clone();
        let mut legal_moves = moves(&current_position);

        if current_depth == 0 { print!("\nanalyze {} legal moves\n", legal_moves.len())}
        if current_depth == 1 { 
            eprint!(".");
        }

        // if can capture king, immediately return
        if let Some(&mov) = legal_moves.iter().find(|mov| mov.captured_piece == Some(Piece::King)) {
            return Some((mov,Self::EVAL_MAX));
        }

        // look forward at half depth to sort the moves
        if current_depth < self.recursion_depth_soft_cap {
            legal_moves.sort_by_key(|&mov| {
                self.input_move(mov);
                let new_depth = (current_depth + 1) * 2;
                let eval = self
                    .suggest_move_internal(new_depth, Self::EVAL_MAX)
                    .map(|(_, eval)| eval)
                    .unwrap_or(0);

                self.roll_back();
                eval
            });
        }


        let result = legal_moves
            .iter()
            .enumerate()
            .scan(None, |best, (i, &mov)| {
                let old_advantage = best.map(|(_, advantage)| advantage).unwrap_or(Self::EVAL_MIN);
                if old_advantage >= prune_threshold { return None }

                let shallow = current_depth < self.recursion_depth_soft_cap;
                let deepening = current_depth < self.recursion_depth_hard_cap;
                let capturing = mov.captured_piece.is_some();

                let mut advantage = if shallow || (deepening && capturing) {
                    self.input_move(mov);
                    let deepening = if i < legal_moves.len() / 3 { 1 } else if i < 3 * legal_moves.len() / 2 { 2 } else { 3 };
                    let adv = self.suggest_move_internal(current_depth + deepening, - old_advantage)
                        .map(|(_, eval)| eval)
                        .unwrap_or(0);
                    self.roll_back();
                    - adv
                } else {
                    - evaluation::evaluate(&apply_move(&current_position, mov))
                };

                // advantage from a move far away in the future gets proportionally less defined
                advantage = advantage / 4 * 3;

                *best = match *best {
                    None => Some((mov, advantage)),
                    Some((_, old_advantage)) if old_advantage < advantage => Some((mov, advantage)),
                    other => other
                };
                *best
            })
            .last();

        // match result {
        //     Some((_, advantage)) => {
        //         match self.evaluation_cache.get(&current_position) {
        //             Some((depth, _)) if *depth <= current_depth => (),
        //             _ => { self.evaluation_cache.insert(current_position, (current_depth, advantage)); },
        //         }
        //     },
        //     None => (),
        // }

        result
    }
}

