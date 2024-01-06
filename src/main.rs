mod position;
mod evaluation;
mod moves;

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
        let mut engine = moves::Engine::new();
        loop {
            let player = engine.current_position().turn;
            match engine.suggest_and_play_move() {
                Some((mov, score)) => {
                    if moves::moves(engine.current_position())
                            .iter()
                            .any(|&mov| mov.captured_piece == Some(position::Piece::King)) {

                        engine.roll_back();
                        println!("{} wins by checkmate!", !player);

                        println!("legal moves:");
                        for mov in moves::moves(engine.current_position()) {
                            engine.input_move(mov);
                            let Some((response, eval)) = engine.suggest_move() else { panic!() };
                            engine.roll_back();
                            match player {
                                position::Color::White => println!("{mov} runs into {response:#} ({})", - eval as f64 / 10000.0),
                                position::Color::Black => println!("{mov:#} runs into {response} ({})", eval as f64 / 10000.0),
                            }
                        }

                        break;
                    }
                    match player {
                        position::Color::White => println!("{player}: {mov} ({})", score as f64 / 10000.0),
                        position::Color::Black => println!("{player}: {mov:#} ({})", - score as f64 / 10000.0),
                    }
                },
                None => { 
                    println!("{player} has no moves available: stalemate");
                    break;
                }
            }

            println!("{}", engine.current_position());
        }
    }
    Ok(())
}
