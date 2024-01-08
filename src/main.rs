mod position;
mod evaluation;
mod human_player;
mod moves;

use human_player::HumanPlayer;

trait Player {
    fn suggest_move(&mut self) -> moves::Move;
    fn input_move(&mut self, mov: moves::Move);
}

impl Player for moves::Engine {
    fn input_move(&mut self, mov: moves::Move) {
        self.input_move(mov);
    }

    fn suggest_move(&mut self) -> moves::Move {
        self.suggest_move().unwrap().0
    }
}

struct Game<'white, 'black> {
    white: &'white mut dyn Player,
    black: &'black mut dyn Player,
    game_history: Vec<position::Position>,
}

enum GameState {
    Ongoing,
    DrawByRepetition,
    StaleMate,
    CheckMate,
}

impl Game<'_, '_> {
    fn current_position(&self) -> &position::Position {
        self.game_history.last().expect("game has positions")
    }

    fn classify_position(&self) -> GameState {
        if self.game_history[0..self.game_history.len() - 1]
            .iter()
            .filter(|&pos| pos == self.current_position())
            .count() >= 3 {
            return GameState::DrawByRepetition;
        }

        let mut mirror_position = self.current_position().clone();
        mirror_position.turn = !mirror_position.turn;

        let king_is_in_check = moves::moves(&mirror_position)
            .iter()
            .any(|&mov| mov.captured_piece == Some(position::Piece::King));

        let king_capture_is_forced = moves::moves(self.current_position())
            .iter()
            .all(|&mov| 
                 moves::moves(&moves::apply_move(self.current_position(), mov))
                     .iter()
                     .any(|&mov| mov.captured_piece == Some(position::Piece::King)));

        match (king_is_in_check, king_capture_is_forced) { 
            (true,  true) => GameState::CheckMate,
            (false, true) => GameState::StaleMate,
            _             => GameState::Ongoing,
        }
    }

    fn play_turn(&mut self) -> bool {
        let mov = match self.current_position().turn {
            position::Color::White => self.white.suggest_move(),
            position::Color::Black => self.black.suggest_move(),
        };

        let position_after_move = moves::apply_move(self.current_position(), mov);

        match self.current_position().turn {
            position::Color::White => println!("{}", mov),
            position::Color::Black => println!("{:#}", mov),
        }

        println!("{position_after_move}");

        self.white.input_move(mov);
        self.black.input_move(mov);
        self.game_history.push(position_after_move);

        match self.classify_position() {
            GameState::CheckMate => {
                println!("{} wins by checkmate", !self.current_position().turn);
                false
            },
            GameState::StaleMate => {
                println!("draw by stalemate");
                false
            },
            GameState::DrawByRepetition => {
                println!("draw by repetition");
                false
            },
            GameState::Ongoing => true
        }
    }

    fn play_full_game(&mut self) {
        while self.play_turn() {}
    }
}

fn main() -> Result<(), String> {

    if std::env::args().any(|arg| arg == "legal-moves") {
        let lines = std::io::stdin().lines().map(|line| line.ok().unwrap()).collect::<Vec<_>>();
        let board = lines.join("\n");
        let start_position = position::Position::try_from(&*board)
            .map_err(|square| format!("parse error at {square}"))?;

        for mov in moves::moves(&start_position) {
            println!("{mov}");
        }
    } else {
        let pos = position::Position::try_from(
"rnbqkbnr
pppppppp
........
........
........
........
PPPPPPPP
RNBQKBNR").expect("hardcoded position is valid");

        let mut game = Game{
            white: &mut HumanPlayer::from_position(&pos),
            black: &mut moves::Engine::from_position(&pos, 4),
            game_history: vec![pos],
        };
        game.play_full_game();
    }

    Ok(())
}
