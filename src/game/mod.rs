mod game_state;
mod board;
mod board_state;

pub use crate::game::game_state::*;
pub use crate::game::board::*;
use crate::base::{Moves, ChessError, ErrorKind, Position, MoveData, Move};
use std::{str, fmt};
use crate::game::board_state::{BoardStates};

#[derive(Clone, Debug)]
pub struct Game {
    latest_state: GameState,
    latest_move: Option<Move>,
    reachable_moves: Moves,
    board_states: BoardStates,
    half_moves_played: usize,
}

impl Game {
    pub fn classic() -> Game {
        let latest_state = GameState::classic();
        Game::from_state(latest_state)
    }

    pub fn from_state(game_state: GameState) -> Game {
        let reachable_moves = game_state.get_reachable_moves();
        Game::from_state_and_reachable_moves(game_state, reachable_moves)
    }

    fn from_state_and_reachable_moves(game_state: GameState, reachable_moves: Moves) -> Game {
        let board_state = game_state.board.encode();
        let turn_by = game_state.turn_by;
        Game {
            latest_state: game_state,
            latest_move: None,
            reachable_moves,
            board_states: BoardStates::new(board_state, turn_by),
            half_moves_played: 0,
        }
    }

    pub fn play(&self, a_move: Move) -> MoveResult {
        let (new_game_state, move_stats) = self.latest_state.do_move(a_move);

        let reachable_moves = match verify_game_state(&new_game_state) {
            Ok(moves) => { moves }
            Err(stopped_reason) => {
                return MoveResult::Stopped(stopped_reason, Box::new(new_game_state));
            }
        };

        let new_board_states: BoardStates = {
            let new_board_state = new_game_state.board.encode();
            let new_turn_by = new_game_state.turn_by;
            let new_board_state_or_stopped_reason =
                self.board_states.add_board_state_and_check_for_draw(
                    new_board_state,
                    new_turn_by,
                    move_stats,
                );

            match new_board_state_or_stopped_reason {
                Ok(board_states) => {board_states},
                Err(stopped_reason) => {
                    return MoveResult::Stopped(stopped_reason, Box::new(new_game_state));
                },
            }
        };

        let new_game = Game {
            latest_state: new_game_state,
            latest_move: Some(a_move),
            reachable_moves,
            board_states: new_board_states,
            half_moves_played: self.half_moves_played + 1,
        };
        MoveResult::Ongoing(Box::new(new_game), move_stats)
    }

    pub fn get_reachable_moves(&self) -> &Moves {
        &self.reachable_moves
    }

    pub fn get_game_state(&self) -> &GameState {
        &self.latest_state
    }

    pub fn is_passive_king_pos(&self, reachable_field: Position) -> bool {
        reachable_field == self.latest_state.get_passive_king_pos()
    }

    pub fn get_fen(&self) -> String {
        let mut fen = self.latest_state.get_fen_part1to4();
        fen.push(' ');
        fen.push_str(self.board_states.count_half_moves_without_progress().to_string().as_str());
        fen.push(' ');
        fen.push_str(((self.half_moves_played / 2) + 1).to_string().as_str());
        fen
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.latest_state)
    }
}

impl str::FromStr for Game {
    type Err = ChessError;

    fn from_str(desc: &str) -> Result<Self, Self::Err> {
        let trimmed_desc = desc.trim();
        if trimmed_desc.is_empty() {
            return Ok(Game::classic())
        }
        let token_iter = trimmed_desc.split(' ');

        // let desc_contains_figures: bool = "♔♕♗♘♖♙♚♛♝♞♜♟".chars().any(|symbol|{desc.contains(symbol)});
        let desc_contains_moves: bool = trimmed_desc.is_empty() || trimmed_desc.contains('-');
        if desc_contains_moves {
            game_by_moves_from_start(token_iter)
        } else {
            game_by_figures_on_board(trimmed_desc)
        }
    }
}

fn game_by_figures_on_board(trimmed_game_config: &str) -> Result<Game, ChessError> {
    let game_state = trimmed_game_config.parse::<GameState>()?;
    if let Err(stopped_reason) = verify_game_state(&game_state) {
        return Err(ChessError {
            msg: format!("game_state {} failed to pass verification: {:?}", game_state, &stopped_reason),
            kind: ErrorKind::HighLevelErr(stopped_reason),
        })
    }
    Ok(Game::from_state(game_state))
}

fn game_by_moves_from_start(token_iter: str::Split<char>) -> Result<Game, ChessError> {
    let mut game = Game::classic();
    for token in token_iter {
        let a_move = token.parse::<Move>()?;
        let move_result = game.play(a_move);
        match move_result {
            MoveResult::Ongoing(new_game, _) => {
                game = *new_game;
            }
            MoveResult::Stopped(reason, _) => {
                return Err(ChessError {
                    msg: format!("game has already ended after move {} because of {:?} in final state {}", a_move, reason, game),
                    kind: ErrorKind::HighLevelErr(reason),
                })
            }
        }
    }
    Ok(game)
}

fn verify_game_state(game_state: &GameState) -> Result<Moves, StoppedReason> {
    let passive_king_pos = game_state.get_passive_king_pos();
    let reachable_moves = game_state.get_reachable_moves();
    if reachable_moves.iter().any(|reachable_move| reachable_move.from_to.to == passive_king_pos) {
        return Err(StoppedReason::KingInCheckAfterMove);
    }
    if !game_state.board.contains_sufficient_material_to_continue() {
        return Err(StoppedReason::InsufficientMaterial);
    }
    Ok(reachable_moves)
}

#[derive(Debug)]
pub enum MoveResult {
    /*
     * bool: was figure taken
     */
    Ongoing(Box<Game>, MoveData),
    Stopped(StoppedReason, Box<GameState>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StoppedReason {
    KingInCheckAfterMove,
    InsufficientMaterial,
    ThreeTimesRepetition,
    NoChangeIn50Moves,
}

//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
    game_config_testing_white, next_move_str, expected_is_insufficient_material,
    case("white ♔e1 ♜f1 ♚e8", "e1f1", true),
    case("white ♔e1 ♜f1 ♘b1 ♘g1 ♚e8 ♞b8 ♞g8", "e1f1", true),
    case("white ♔e1 ♜f1 ♘b1 ♘g1 ♚e8 ♞b8 ♞g8 ♞h8", "e1f1", false),
    case("white ♔e1 ♜f1 ♚e8 ♞b8 ♞g8 ♞h8", "e1f1", false),
    case("white ♔e1 ♜f1 ♗b1 ♚e8 ♞b8 ♞g8", "e1f1", true),
    case("white ♔e1 ♜f1 ♗b1 ♚e8 ♝g8", "e1f1", true),
    case("white ♔e1 ♜f1 ♗b1 ♘g8 ♚e8", "e1f1", false),
    case("white ♔e1 ♜f1 ♗b1 ♗g8 ♚e8", "e1f1", false),
    case("white ♔e1 ♖b1 ♚e8", "e1f1", false),
    case("white ♔e1 ♛b1 ♚e8", "e1f1", false),
    case("white ♔e1 ♙b2 ♚e8", "e1f1", false),
    case("white ♔e1 ♙a2 ♙b2 ♙c2 ♙d2 ♙e2 ♙f2 ♚e8", "e1f1", false),
    case("white ♔a1 ♟a2 ♚a3", "a1a2", false), // because king is in check after move
    ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_game_ends_bc_insufficient_material(
        game_config_testing_white: &str,
        next_move_str: &str,
        expected_is_insufficient_material: bool,
    ) {
        fn is_insufficient_material(move_result: MoveResult) -> bool {
            matches!(move_result, MoveResult::Stopped(StoppedReason::InsufficientMaterial, _))
        }

        let game = game_config_testing_white.parse::<Game>().unwrap();
        let next_move = next_move_str.parse::<Move>().unwrap();
        let move_result = game.play(next_move);
        assert_eq!(is_insufficient_material(move_result), expected_is_insufficient_material);
    }

    #[rstest(
    game_config, expected_stop_reason,
    case("white ♔h8 ♚f8 ♞a7", StoppedReason::InsufficientMaterial),
    case("black ♔h8 ♚f8 ♛g7", StoppedReason::KingInCheckAfterMove),
    case("b1-c3 b8-c6 c3-b1 c6-b8 b1-c3 b8-c6 c3-b1 c6-b8 b1-c3", StoppedReason::ThreeTimesRepetition),
    case("b1-c3 b8-c6 c3-b1 c6-b8 b1-c3 b8-c6 c3-b1 c6-b8", StoppedReason::ThreeTimesRepetition),
    ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_parse_stopped_game(
        game_config: &str,
        expected_stop_reason: StoppedReason,
    ) {
        // pub enum StoppedReason {
        //     KingInCheckAfterMove,
        //     InsufficientMaterial,
        //     ThreeTimesRepetition,
        //     NoChangeIn50Moves,
        // }
        match game_config.parse::<Game>() {
            Err(err) => {
                if let ErrorKind::HighLevelErr(actual_stopped_reason) = err.kind {
                    assert_eq!(actual_stopped_reason, expected_stop_reason);
                } else {
                    panic!("expected HighLevelErr but got {}", err);
                }
            }
            Ok(_) => {
                panic!("expected HighLevelErr but got game");
            }
        };
    }

    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
    game_config, expected_fen,
    case("", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
    case("e2-e4", "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"),
    case("e2-e4 e7-e5", "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2"),
    case("b1-a3 g8-h6 g1-h3", "rnbqkb1r/pppppppp/7n/8/8/N6N/PPPPPPPP/R1BQKB1R b KQkq - 3 2"),
    case("b1-a3 g8-h6 a1-b1", "rnbqkb1r/pppppppp/7n/8/8/N7/PPPPPPPP/1RBQKBNR b Kkq - 3 2"),
    case("b1-a3 g8-h6 a1-b1 h8-g8", "rnbqkbr1/pppppppp/7n/8/8/N7/PPPPPPPP/1RBQKBNR w Kq - 4 3"),
    case("white ♔d1 ♖h1 ♚e8", "4k3/8/8/8/8/8/8/3K3R w - - 0 1"),
    case("black ♖a1 ♔e1 ♖h1 ♜a8 ♚e8 ♜h8", "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1"),
    ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_get_fen(
        game_config: &str,
        expected_fen: &str,
    ) {
        let game = game_config.parse::<Game>().unwrap();
        let actual_fen = game.get_fen();
        assert_eq!(actual_fen, String::from(expected_fen));
    }
}