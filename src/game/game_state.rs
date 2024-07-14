use tinyvec::*;
use std::{fmt,str};
use crate::base::a_move::{FromTo, Move, MoveData, Moves, MoveType, PromotionType};
use crate::base::a_move::CastlingType::{KingSide, QueenSide};
use crate::base::color::Color;
use crate::base::direction::Direction;
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::position::Position;
use crate::base::util::Disallowable;
use crate::figure::figure::{Figure, FigureAndPosition, FigureType};
use crate::game::board::{Board, CaptureInfoOption};

#[derive(Clone, Debug)]
pub struct GameState {
    pub board: Board,
    pub turn_by: Color,
    white_king_pos: Position,
    black_king_pos: Position,
    pub en_passant_intercept_pos: Option<Position>,
    pub is_white_queen_side_castling_still_allowed: Disallowable,
    pub is_white_king_side_castling_still_allowed: Disallowable,
    pub is_black_queen_side_castling_still_allowed: Disallowable,
    pub is_black_king_side_castling_still_allowed: Disallowable,
    moves_played: Vec<Move>,
}

impl GameState {
    pub fn classic() -> GameState {
        GameState {
            board: Board::classic(),
            turn_by: Color::White,
            white_king_pos: "e1".parse::<Position>().ok().unwrap(),
            black_king_pos: "e8".parse::<Position>().ok().unwrap(),
            en_passant_intercept_pos: None,
            is_white_queen_side_castling_still_allowed: Disallowable::new(true),
            is_white_king_side_castling_still_allowed: Disallowable::new(true),
            is_black_queen_side_castling_still_allowed: Disallowable::new(true),
            is_black_king_side_castling_still_allowed: Disallowable::new(true),
            moves_played: Vec::new(),
        }
    }


    pub fn from_manual_config(
        turn_by: Color,
        en_passant_intercept_pos: Option<Position>,
        positioned_figures: Vec<FigureAndPosition>
    ) -> Result<GameState, ChessError> {
        let mut board = Board::empty();
        let mut opt_white_king_pos: Option<Position> = None;
        let mut opt_black_king_pos: Option<Position> = None;

        for figure_and_pos in positioned_figures {
            let field_was_already_in_use = board.set_figure(figure_and_pos.pos, figure_and_pos.figure);
            if field_was_already_in_use.is_some() {
                return Err(ChessError{
                    msg: format!("multiple figures placed on {}", figure_and_pos.pos),
                    kind: ErrorKind::IllegalConfig
                })
            }
            match figure_and_pos.figure.fig_type {
                FigureType::Pawn => {
                    let pawn_pos_row = figure_and_pos.pos.row;
                    if pawn_pos_row==0 || pawn_pos_row==7 {
                        return Err(ChessError{
                            msg: format!("can't place a pawn on {}", figure_and_pos.pos),
                            kind: ErrorKind::IllegalConfig
                        })
                    }
                },
                FigureType::King => {
                    match figure_and_pos.figure.color {
                        Color::White => {
                            if opt_white_king_pos.is_some() {
                                return Err(ChessError{
                                    msg: format!("can't place a pawn on {}. That row isn't reachable for a pawn.", figure_and_pos.pos),
                                    kind: ErrorKind::IllegalConfig
                                })
                            }
                            opt_white_king_pos = Some(figure_and_pos.pos);
                        },
                        Color::Black => {
                            if opt_black_king_pos.is_some() {
                                return Err(ChessError{
                                    msg: format!("can't place a pawn on {}. That row isn't reachable for a pawn.", figure_and_pos.pos),
                                    kind: ErrorKind::IllegalConfig
                                })
                            }
                            opt_black_king_pos = Some(figure_and_pos.pos);
                        },
                    }
                },
                _ => {},
            };
        }

        // check en-passant
        if let Some(en_passant_pos) = en_passant_intercept_pos {
            let (
                expected_row,
                expected_row_in_text,
                forward_dir,
            ) = match turn_by {
                Color::White => {
                    (5_i8, 6_i8, Direction::Down)
                }
                Color::Black => {
                    (2_i8, 3_i8, Direction::Up)
                }
            };
            if en_passant_pos.row != expected_row {
                return Err(ChessError {
                    msg: format!("it's {}'s turn so the en-passant position has to be on the {}th row but it's {}.", turn_by, expected_row_in_text, en_passant_pos),
                    kind: ErrorKind::IllegalConfig,
                })
            }
            let forward_pawn_pos = en_passant_pos.step(forward_dir).unwrap();
            let mut contains_correct_pawn = false;
            if let Some(forward_figure) = board.get_figure(forward_pawn_pos) {
                if forward_figure.fig_type==FigureType::Pawn && forward_figure.color!=turn_by {
                    contains_correct_pawn = true;
                }
            }
            if !contains_correct_pawn {
                return Err(ChessError {
                    msg: format!("since {} is an en-passant pos, there should be a {} pawn on {} but isn't.", en_passant_pos, turn_by.toggle(), forward_pawn_pos),
                    kind: ErrorKind::IllegalConfig,
                })
            }

            let backward_empty_pos = en_passant_pos.step(forward_dir.reverse()).unwrap();
            if !board.is_empty(backward_empty_pos) {
                return Err(ChessError {
                    msg: format!("since {} is an en-passant pos, the position behind it ({}) should be empty but isn't.", en_passant_pos, backward_empty_pos),
                    kind: ErrorKind::IllegalConfig,
                })
            }
        }

        let white_king_pos = match opt_white_king_pos {
            Some(pos) => pos,
            None => {
                return Err(ChessError{
                    msg: "no white king configured".to_string(),
                    kind: ErrorKind::IllegalConfig
                })
            },
        };
        let black_king_pos = match opt_black_king_pos {
            Some(pos) => pos,
            None => {
                return Err(ChessError{
                    msg: "no white king configured".to_string(),
                    kind: ErrorKind::IllegalConfig
                })
            },
        };

        fn board_contains_rook_at(pos: Position, color: Color, board: &Board) -> bool {
            if let Some(figure) = board.get_figure(pos) {
                figure.fig_type==FigureType::Rook && figure.color==color
            } else {
                false
            }
        }

        let is_white_king_on_starting_pos = white_king_pos == WHITE_KING_STARTING_POS;
        let is_black_king_on_starting_pos = black_king_pos == BLACK_KING_STARTING_POS;

        let is_white_queen_side_rook_on_starting_pos = board_contains_rook_at(
            WHITE_QUEEN_SIDE_ROOK_STARTING_POS, Color::White, &board,
        );
        let is_white_king_side_rook_on_starting_pos = board_contains_rook_at(
            WHITE_KING_SIDE_ROOK_STARTING_POS, Color::White, &board,
        );
        let is_black_queen_side_rook_on_starting_pos = board_contains_rook_at(
            BLACK_QUEEN_SIDE_ROOK_STARTING_POS, Color::Black, &board,
        );
        let is_black_king_side_rook_on_starting_pos = board_contains_rook_at(
            BLACK_KING_SIDE_ROOK_STARTING_POS, Color::Black, &board,
        );
        let is_white_queen_side_castling_possible = Disallowable::new(is_white_king_on_starting_pos && is_white_queen_side_rook_on_starting_pos);
        let is_white_king_side_castling_possible = Disallowable::new(is_white_king_on_starting_pos && is_white_king_side_rook_on_starting_pos);
        let is_black_queen_side_castling_possible = Disallowable::new(is_black_king_on_starting_pos && is_black_queen_side_rook_on_starting_pos);
        let is_black_king_side_castling_possible = Disallowable::new(is_black_king_on_starting_pos && is_black_king_side_rook_on_starting_pos);

        let game_state = GameState {
            board,
            turn_by,
            white_king_pos,
            black_king_pos,
            en_passant_intercept_pos,
            is_white_queen_side_castling_still_allowed: is_white_queen_side_castling_possible,
            is_white_king_side_castling_still_allowed: is_white_king_side_castling_possible,
            is_black_queen_side_castling_still_allowed: is_black_queen_side_castling_possible,
            is_black_king_side_castling_still_allowed: is_black_king_side_castling_possible,
            moves_played: Vec::new(),
        };

        Ok(game_state)
    }

    /**
     * returns true if a_move.from points to a pawn and a_move.to is on the first or last row of the board
     * (but doesn't check if the move is actually legal)
     */
    pub fn looks_like_pawn_promotion_move(&self, a_move: FromTo) -> bool {
        let Some(Figure{fig_type: FigureType::Pawn, color: _}) = self.board.get_figure(a_move.from) else {
            return false;
        };
        let pawn_to_row = a_move.to.row;
        (pawn_to_row == 7) || (pawn_to_row == 0)
    }

    /**
     * returns true if a_move.from points to a king and a_move.to points to rook of the same color
     * (but doesn't check if the move is actually legal)
     * returns error if a_move.to is the classic target field of the king. since that move can be ambiguous
     * in a chess960 context.
     */
    pub fn looks_like_castling(&self, a_move: FromTo) -> Result<bool, ChessError> {
        let Some(Figure{fig_type: FigureType::King, color: _}) = self.board.get_figure(a_move.from) else {
            return Ok(false);
        };
        if let Some(Figure{fig_type: FigureType::Rook, color: rook_color}) = self.board.get_figure(a_move.to) {
            if rook_color==self.turn_by {
                return Ok(true);
            };
        };
        let ground_row = self.turn_by.get_ground_row();
        if a_move.from.get_row_distance(a_move.to) > 1 && a_move.from.row == ground_row && a_move.to.row == ground_row {
            return Err(ChessError{
                msg: "It looks like you're trying to castle by pointing to the final position of the king. Point to the rook you're castling with instead!".to_string(),
                kind: ErrorKind::IllegalFormat,
            })
        }
        return Ok(false);
    }

    // TODO change return type to Result<(GameState, Move), ChessError>
    pub fn do_move(&self, next_move: Move) -> (GameState, MoveData) {
        let from = next_move.from_to.from;
        let to = next_move.from_to.to;

        debug_assert!(
            to != self.white_king_pos && to != self.black_king_pos,
            "move {} would capture a king on game {}", next_move, self.board
        );
        debug_assert!(
            self.board.contains_figure(self.white_king_pos, FigureType::King, Color::White),
            "couldn't find white king at white_king_pos {} on board {} (next_move {})", self.white_king_pos, self.board, next_move
        );
        debug_assert!(
            self.board.contains_figure(self.black_king_pos, FigureType::King, Color::Black),
            "couldn't find black king at black_king_pos {} on board {} (next_move {})", self.black_king_pos, self.board, next_move
        );

        let mut new_board = self.board.clone();
        let moving_figure: Figure = self.board.get_figure(from).unwrap();

        let mut new_is_white_queen_side_castling_allowed = self.is_white_queen_side_castling_still_allowed;
        let mut new_is_white_king_side_castling_allowed = self.is_white_king_side_castling_still_allowed;
        let mut new_is_black_queen_side_castling_allowed = self.is_black_queen_side_castling_still_allowed;
        let mut new_is_black_king_side_castling_allowed = self.is_black_king_side_castling_still_allowed;

        {
            if from == WHITE_QUEEN_SIDE_ROOK_STARTING_POS || to == WHITE_QUEEN_SIDE_ROOK_STARTING_POS {
                new_is_white_queen_side_castling_allowed.disallow()
            }
            if from == WHITE_KING_SIDE_ROOK_STARTING_POS || to == WHITE_KING_SIDE_ROOK_STARTING_POS {
                new_is_white_king_side_castling_allowed.disallow()
            }
            if from == BLACK_QUEEN_SIDE_ROOK_STARTING_POS || to == BLACK_QUEEN_SIDE_ROOK_STARTING_POS {
                new_is_black_queen_side_castling_allowed.disallow()
            }
            if from == BLACK_KING_SIDE_ROOK_STARTING_POS || to == BLACK_KING_SIDE_ROOK_STARTING_POS {
                new_is_black_king_side_castling_allowed.disallow()
            }
        }

        let (
            new_white_king_pos,
            new_black_king_pos,
            new_en_passant_intercept_pos,
            move_stats,
        ) = match moving_figure.fig_type {
            FigureType::King => {
                let is_castling = match new_board.get_figure(to) {
                    Some(Figure{fig_type: FigureType::Rook, color: rook_color }) => {
                        rook_color == moving_figure.color
                    }
                    _ => false,
                };

                let (effective_king_move, figure_captured, castling_rook_move) = if is_castling {
                    let (king_move, rook_move) = do_castling_move(&mut new_board, next_move.from_to, moving_figure.color);
                    (king_move, None, Some(rook_move))
                } else {
                    let capture_info = do_normal_move(&mut new_board, next_move.from_to);
                    (next_move.from_to, capture_info.get_captured_figure_type(), None)
                };

                let king_move_stats = {
                    let mut stats = MoveData::new(next_move.from_to, FigureType::King, figure_captured);
                    let move_type = if let Some(rook_move) = castling_rook_move {
                        let castling_type = if rook_move.to.column==3 {
                            QueenSide
                        } else {
                            KingSide
                        };
                        MoveType::Castling { c_type: castling_type, king_move: effective_king_move, rook_move: rook_move }
                    } else {
                        MoveType::Normal
                    };
                    stats.move_type = move_type.into();
                    stats
                };


                match moving_figure.color {
                    Color::White => {
                        new_is_white_queen_side_castling_allowed.disallow();
                        new_is_white_king_side_castling_allowed.disallow();
                        (
                            effective_king_move.to,
                            self.black_king_pos,
                            None,
                            king_move_stats,
                        )
                    }
                    Color::Black => {
                        new_is_black_queen_side_castling_allowed.disallow();
                        new_is_black_king_side_castling_allowed.disallow();
                        (
                            self.white_king_pos,
                            effective_king_move.to,
                            None,
                            king_move_stats,
                        )
                    }
                }
            },
            FigureType::Pawn => {
                fn compute_pawn_move_type(this: &GameState, pawn_move: Move) -> PawnMoveType {
                    if let Some(promotion_type) = pawn_move.promotion_type {
                        return PawnMoveType::Promotion(promotion_type);
                    };
                    let pawn_move_from = pawn_move.from_to.from;
                    let pawn_move_to   = pawn_move.from_to.to;
                    if pawn_move_from.get_row_distance(pawn_move_to) == 2 {
                        return PawnMoveType::DoubleStep
                    }
                    if let Some(en_passant_pos) = this.en_passant_intercept_pos {
                        if pawn_move_to == en_passant_pos {
                            return PawnMoveType::EnPassantIntercept
                        }
                    }
                    PawnMoveType::SingleStep
                }
                fn handle_pawn_promotion_after_move(new_board: &mut Board, pawn_move: Move, pawn_color: Color) {
                    if let Some(promo_type) = pawn_move.promotion_type {
                        new_board.set_figure(
                            pawn_move.from_to.to,
                            Figure{ fig_type: promo_type.get_figure_type(), color: pawn_color }
                        );
                    }
                }

                match compute_pawn_move_type(self, next_move) {
                    PawnMoveType::Promotion(promotion_type) => {
                        let capture_info: CaptureInfoOption = do_normal_move(&mut new_board, next_move.from_to);
                        handle_pawn_promotion_after_move(&mut new_board, next_move, self.turn_by);
                        let stats = MoveData::new_pawn_promotion(next_move.from_to, capture_info.get_captured_figure_type(), promotion_type);
                        (
                            self.white_king_pos, self.black_king_pos,
                            None,
                            stats,
                        )
                    },
                    PawnMoveType::SingleStep => {
                        let capture_info: CaptureInfoOption = do_normal_move(&mut new_board, next_move.from_to);
                        handle_pawn_promotion_after_move(&mut new_board, next_move, self.turn_by);
                        let stats = MoveData::new(next_move.from_to, FigureType::Pawn, capture_info.get_captured_figure_type());
                        (
                            self.white_king_pos, self.black_king_pos,
                            None,
                            stats,
                        )
                    },
                    PawnMoveType::DoubleStep => {
                        do_normal_move(&mut new_board, next_move.from_to);
                        let stats = MoveData::new(next_move.from_to, FigureType::Pawn, None);
                        (
                            self.white_king_pos, self.black_king_pos,
                            Some(Position::new_unchecked(
                                to.column,
                                (from.row + to.row) / 2,
                            )),
                            stats,
                        )
                    },
                    PawnMoveType::EnPassantIntercept => {
                        do_en_passant_move(&mut new_board, next_move.from_to);
                        let a_move = MoveData::new_en_passant(next_move.from_to);
                        (
                            self.white_king_pos, self.black_king_pos,
                            None,
                            a_move,
                        )
                    },
                }
            },
            _ => {
                let capture_info = do_normal_move(&mut new_board, next_move.from_to);
                (
                    self.white_king_pos,
                    self.black_king_pos,
                    None,
                    MoveData::new(next_move.from_to, FigureType::Pawn, capture_info.get_captured_figure_type()),
                )
            },
        };

        (GameState {
            board: new_board,
            turn_by: self.turn_by.toggle(),
            white_king_pos: new_white_king_pos,
            black_king_pos: new_black_king_pos,
            en_passant_intercept_pos: new_en_passant_intercept_pos,
            is_white_queen_side_castling_still_allowed: new_is_white_queen_side_castling_allowed,
            is_white_king_side_castling_still_allowed: new_is_white_king_side_castling_allowed,
            is_black_queen_side_castling_still_allowed: new_is_black_queen_side_castling_allowed,
            is_black_king_side_castling_still_allowed: new_is_black_king_side_castling_allowed,
            moves_played: {
                let mut new_moves_played = self.moves_played.clone();
                new_moves_played.push(next_move);
                new_moves_played
            },
        },
         move_stats,
        )
    }

    pub fn get_reachable_moves(&self) -> Moves {
        let mut move_collector: Moves = tiny_vec!();
        let figures_of_color_with_pos: [Option<(Figure, Position)>; 16] =
            self.board.get_all_figures_of_color(self.turn_by);

        for figure_of_color in figures_of_color_with_pos {
            match figure_of_color {
                Some((figure, pos)) => {
                    figure.for_reachable_moves(pos, self, &mut move_collector);
                },
                None => {
                    break;
                }
            }
        }

        move_collector
    }

    pub fn get_passive_kings_pos(&self) -> Position {
        match self.turn_by {
            Color::White => self.black_king_pos,
            Color::Black => self.white_king_pos,
        }
    }

    pub fn get_passive_king_pos(&self) -> Position {
        match self.turn_by {
            Color::Black => {self.white_king_pos}
            Color::White => {self.black_king_pos}
        }
    }

    pub fn get_active_king(&self) -> Position {
        if self.turn_by==Color::White {
            self.white_king_pos
        } else {
            self.black_king_pos
        }
    }

    pub fn get_fen_part1to4(&self) -> String {
        let mut fen_part1to4 = self.board.get_fen_part1();
        fen_part1to4.push(' ');
        fen_part1to4.push(self.turn_by.get_fen_char());
        fen_part1to4.push(' ');
        let white_king_castling = self.is_white_king_side_castling_still_allowed.is_still_allowed();
        let white_queen_castling = self.is_white_queen_side_castling_still_allowed.is_still_allowed();
        let black_king_castling = self.is_black_king_side_castling_still_allowed.is_still_allowed();
        let black_queen_castling = self.is_black_queen_side_castling_still_allowed.is_still_allowed();
        if white_king_castling { fen_part1to4.push('K'); }
        if white_queen_castling { fen_part1to4.push('Q'); }
        if black_king_castling { fen_part1to4.push('k'); }
        if black_queen_castling { fen_part1to4.push('q'); }
        if !(white_king_castling || white_queen_castling || black_king_castling || black_queen_castling) {
            fen_part1to4.push('-');
        }
        fen_part1to4.push(' ');
        match self.en_passant_intercept_pos {
            None => { fen_part1to4.push('-');}
            Some(pos) => { fen_part1to4.push_str(format!("{}", pos).as_str());}
        }
        fen_part1to4
    }

    pub fn get_moves_played(&self) -> Vec<Move> {
        // let debug_format = format!("{:?}", self.moves_played);
        // // remove the embracing '[' and ']'
        // let last_char_index = debug_format.len()-1;
        // debug_format[1..last_char_index].to_string()
        self.moves_played.clone()
    }
}

impl str::FromStr for GameState {
    type Err = ChessError;

    fn from_str(desc: &str) -> Result<Self, Self::Err> {
        let trimmed_desc = desc.trim();
        if trimmed_desc.is_empty() {
            return Ok(GameState::classic())
        }
        let token_iter = trimmed_desc.split(' ');

        // let desc_contains_figures: bool = "♔♕♗♘♖♙♚♛♝♞♜♟".chars().any(|symbol|{desc.contains(symbol)});
        let desc_contains_moves: bool = trimmed_desc.is_empty() || !(trimmed_desc.starts_with("white") || trimmed_desc.starts_with("black"));
        println!("'{desc_contains_moves}', '{trimmed_desc}'");
        if desc_contains_moves {
            game_by_moves_from_start(token_iter)
        } else {
            game_by_figures_on_board(token_iter)
        }
    }
}

fn game_by_moves_from_start(token_iter: str::Split<char>) -> Result<GameState, ChessError> {
    let mut game_state = GameState::classic();
    for token in token_iter {
        let basic_move = token.parse::<Move>()?;
        let (new_game_state, _) = game_state.do_move(basic_move);
        game_state = new_game_state;
    }
    Ok(game_state)
}

fn game_by_figures_on_board(mut token_iter: str::Split<char>) -> Result<GameState, ChessError> {
    let first_token = token_iter.next().unwrap();
    let turn_by = match first_token {
        "white" => Color::White,
        "black" => Color::Black,
        _ => {
            return Err(ChessError {
                msg: format!("the first token has to be either 'white' or 'black' but was {}", first_token),
                kind: ErrorKind::IllegalConfig,
            })
        },
    };

    let mut positioned_figures: Vec<FigureAndPosition> = vec![];
    let mut opt_en_passant_pos: Option<Position> = None;

    for token in token_iter {
        // tokens should either start with a figure char (from "♔♕♗♘♖♙♚♛♝♞♜♟") or E (for en-passant)
        // followed by a position between "a1" and "h8"
        if let Some(stripped_token) = token.strip_prefix('E') {
            let en_passant_pos = stripped_token.parse::<Position>()?;
            if let Some(old_en_passant_pos) = opt_en_passant_pos {
                return Err(ChessError {
                    msg: format!("there are two en-passant tokens present (on {} and {}) but only one is allowed.", old_en_passant_pos, en_passant_pos),
                    kind: ErrorKind::IllegalConfig,
                })
            }
            opt_en_passant_pos = Some(en_passant_pos);
        } else {
            let figure_and_pos = token.parse::<FigureAndPosition>()?;
            positioned_figures.push(figure_and_pos);
        }
    }

    let game_state = GameState::from_manual_config(turn_by, opt_en_passant_pos, positioned_figures)?;
    Ok(game_state)
}

/**
 * returns the figure that was caught (if any) and the position it was caught on
 */
fn do_normal_move(
    new_board: &mut Board,
    next_move: FromTo,
) -> CaptureInfoOption {
    let moving_figure: Figure = new_board.get_figure(next_move.from).expect("field the figure moves from is empty");
    new_board.clear_field(next_move.from);
    new_board.set_figure(next_move.to, moving_figure)
}

/**
 * this function assumes that the king move given this function actually moves to the position of the rook
 * (this makes castling moves distinguishable from normal king moves in all initial positions of chess960,
 * e.g. think of initial position with king on f1 and rook on h1: would move f1g1 come with intend to castle or not?)
 * returns the effective move of king and rook
 * e.g. giving classic initial position, then king_move: e1h1 would return (e1g1, h1f1)
 */
fn do_castling_move(
    new_board: &mut Board,
    king_move: FromTo,
    king_color: Color,
) -> (FromTo, FromTo) {
    new_board.clear_field(king_move.from);
    new_board.clear_field(king_move.to);
    let move_row = king_move.to.row;
    let castling_type = if king_move.to.column > king_move.from.column {
        KingSide
    } else {
        QueenSide
    };
    let (king_to_pos, rook_to_pos) = if castling_type== KingSide {
        let rook_to_pos = Position::new_unchecked(5, move_row);
        let king_to_pos = Position::new_unchecked(6, move_row);
        new_board.set_figure(king_to_pos, Figure{ fig_type: FigureType::King, color: king_color });
        new_board.set_figure(rook_to_pos, Figure{ fig_type: FigureType::Rook, color: king_color });
        (king_to_pos, rook_to_pos)
    } else {
        let rook_to_pos = Position::new_unchecked(3, move_row);
        let king_to_pos = Position::new_unchecked(2, move_row);
        new_board.set_figure(king_to_pos, Figure{ fig_type: FigureType::King, color: king_color });
        new_board.set_figure(rook_to_pos, Figure{ fig_type: FigureType::Rook, color: king_color });
        (king_to_pos, rook_to_pos)
    };

    (FromTo::new(king_move.from, king_to_pos), FromTo::new(king_move.to, rook_to_pos))
}

fn do_en_passant_move(
    new_board: &mut Board,
    next_move: FromTo,
) -> CaptureInfoOption {
    do_normal_move(new_board, next_move);
    let double_stepped_pawn_pos =
        Position::new_unchecked(next_move.to.column, next_move.from.row);
    let pawn_captured = new_board.get_figure(double_stepped_pawn_pos).unwrap();
    new_board.clear_field(double_stepped_pawn_pos);
    CaptureInfoOption::from_some(pawn_captured, double_stepped_pawn_pos)
}

enum PawnMoveType {
    SingleStep, DoubleStep, EnPassantIntercept, Promotion(PromotionType),
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}'s turn", self.turn_by)?;
        writeln!(f, "{}", self.board)
    }
}

pub static WHITE_KING_STARTING_POS: Position = Position::new_unchecked(4, 0);
static WHITE_KING_SIDE_ROOK_STARTING_POS: Position = Position::new_unchecked(7, 0);
static WHITE_QUEEN_SIDE_ROOK_STARTING_POS: Position = Position::new_unchecked(0, 0);
pub static BLACK_KING_STARTING_POS: Position = Position::new_unchecked(4, 7);
static BLACK_KING_SIDE_ROOK_STARTING_POS: Position = Position::new_unchecked(7, 7);
static BLACK_QUEEN_SIDE_ROOK_STARTING_POS: Position = Position::new_unchecked(0, 7);

//------------------------------Tests------------------------

#[cfg(test)]
mod tests {
    impl GameState {
        pub fn toggle_colors(&self) -> GameState {
            fn toggle_figures_on_board_to(color: Color, figure_array: [Option<(FigureType, Position)>; 16], board: &mut Board) {
                for opt_figure_type_and_pos in figure_array.iter() {
                    if let Some((figure_type, pos)) = opt_figure_type_and_pos {
                        board.set_figure(pos.toggle_row(), Figure{ fig_type: *figure_type, color });
                    } else {
                        break;
                    }
                }
            }
            let mut toggled_board = Board::empty();
            let (array_of_opt_white_figures, array_of_opt_black_figures) = self.board.get_white_and_black_figures();
            toggle_figures_on_board_to(Color::Black, array_of_opt_white_figures, &mut toggled_board);
            toggle_figures_on_board_to(Color::White, array_of_opt_black_figures, &mut toggled_board);

            GameState {
                board: toggled_board,
                turn_by: self.turn_by.toggle(),
                white_king_pos: self.black_king_pos.toggle_row(),
                black_king_pos: self.white_king_pos.toggle_row(),
                en_passant_intercept_pos: self.en_passant_intercept_pos.map(|pos|{pos.toggle_row()}),
                is_white_queen_side_castling_still_allowed: self.is_black_queen_side_castling_still_allowed,
                is_white_king_side_castling_still_allowed: self.is_black_king_side_castling_still_allowed,
                is_black_queen_side_castling_still_allowed: self.is_white_queen_side_castling_still_allowed,
                is_black_king_side_castling_still_allowed: self.is_white_king_side_castling_still_allowed,
                moves_played: toggle_rows(&self.moves_played),
            }
        }
    }

    use itertools::Itertools;
    use super::*;
    use rstest::*;
    use crate::base::a_move::toggle_rows;
    use crate::base::color::Color;

    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
        _game_state,
        case(""),
        case("e2e4 e7e5"),
        case("white ♖a1 ♔e1 ♖h1 ♙a2 ♜h2 ♚e8"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_game_from_str(
        _game_state: GameState,
    ) {}

    // TODO: use to check for checkmate after the final move or delete
    // //♔♕♗♘♖♙♚♛♝♞♜♟
    //
    // #[rstest(
    //     game_state, expected_nr_of_reachable_moves,
    //     case("", 20),
    //     case("e2e4 e7e5", 29),
    //     case("e2e4 a7a6", 30),
    //     case("e2e4 b7b5", 29),
    //     case("a2a4 a7a6 a4a5 b7b5", 22), // en-passant
    //     case("white ♔a1 ♙b5 ♟a6 Ec6 ♟c5 ♚e8", 6), // en-passant
    //     case("white ♖a2 ♔e2 ♖h2 ♚e8", 27), // no castling
    //     case("white ♖a1 ♔e1 ♖h1 ♚e8", 26), // castling
    //     case("white ♖a1 ♔e1 ♖h1 ♙a2 ♜h2 ♚e8", 15), // castling
    //     case("white ♔a1 ♚c1", 3), // king can be caught
    //     case("white ♔a1 ♚b1", 3), // king can be caught
    //     ::trace //This leads to the arguments being printed in front of the test result.
    // )]
    // fn test_get_reachable_moves(
    //     game_state: GameState,
    //     expected_nr_of_reachable_moves: usize,
    // ) {
    //     let white_nr_of_reachable_moves = game_state.get_reachable_moves().len();
    //     assert_eq!(white_nr_of_reachable_moves, expected_nr_of_reachable_moves, "nr of reachable moves");
    //
    //     let black_nr_of_reachable_moves = game_state.toggle_colors().get_reachable_moves().len();
    //     assert_eq!(black_nr_of_reachable_moves, expected_nr_of_reachable_moves, "nr of reachable moves");
    // }

    //♔♕♗♘♖♙♚♛♝♞♜♟

    #[rstest(
        game_state, next_move_str, expected_catches_figure,
        case("white ♔e1 ♖h1 ♙a2 ♜h2 ♚e8", "e1d1", false),
        case("white ♔e1 ♖h1 ♙a2 ♜h2 ♚e8", "e1g1", false),
        case("white ♔e1 ♖h1 ♙a2 ♜h2 ♚e8", "a2a3", false),
        case("white ♔e1 ♖h1 ♙a2 ♜h2 ♚e8", "a2a4", false),
        case("white ♔e1 ♖h1 ♙a2 ♜h2 ♚e8", "h1h2", true),
        case("b2b4 a7a6 b4b5 c7c5", "b5c6", true),
        case("b2b4 a7a6 b4b5 c7c5", "b5a6", true),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_do_move_catches_figure(
        game_state: GameState,
        next_move_str: &str,
        expected_catches_figure: bool,
    ) {
        let white_move = next_move_str.parse::<Move>().unwrap();
        let ( _, move_stats) = game_state.do_move(white_move);
        assert_eq!(move_stats.did_catch_figure(), expected_catches_figure, "white catches figure");


        let toggled_game_state = game_state.toggle_colors();
        let ( _, move_stats) = toggled_game_state.do_move(white_move.toggle_rows());
        assert_eq!(move_stats.did_catch_figure(), expected_catches_figure, "black catches figure");
    }

    #[test]
    fn test_game_state_toggle_colors() {
        let game_state = "white ♔b1 ♜h2 Eh6 ♟h5 ♚g7".parse::<GameState>().unwrap();
        let white_move = "b1c1".parse::<Move>().unwrap();
        assert_eq!(game_state.turn_by, Color::White);
        assert_eq!(game_state.get_passive_king_pos(), "g7".parse::<Position>().unwrap());
        assert_eq!(game_state.en_passant_intercept_pos.unwrap(), "h6".parse::<Position>().unwrap());
        // do_move includes some runtime validation
        game_state.do_move(white_move);


        let toggled_game_state = game_state.toggle_colors();
        assert_eq!(toggled_game_state.turn_by, Color::Black);
        assert_eq!(toggled_game_state.get_passive_king_pos(), "g2".parse::<Position>().unwrap(), "game_state {}", &toggled_game_state);
        assert_eq!(toggled_game_state.en_passant_intercept_pos.unwrap(), "h3".parse::<Position>().unwrap(), "game_state {}", &toggled_game_state);
        toggled_game_state.do_move(white_move.toggle_rows());
    }

    #[rstest(
        game_state, expected_color,
        case("black ♔b6 ♙a7 ♚a8", Color::Black),
        case("white ♔h8 ♚f8 ♜e7 ♟e6 ♟d7", Color::White),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_turn_by(
        game_state: GameState,
        expected_color: Color,
    ) {
        assert_eq!(game_state.turn_by, expected_color);
    }

    #[rstest(
        game_state, promoting_move,
        case("white ♔b6 ♙a7 ♚h6", "a7a8Q"),
        case("white ♔b6 ♙a7 ♚h6", "a7a8R"),
        case("white ♔b6 ♙a7 ♚h6", "a7a8K"),
        case("white ♔b6 ♙a7 ♚h6", "a7a8B"),
        case("white ♔b6 ♙a7 ♞b8 ♚h6", "a7b8Q"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_pawn_promo_works(
        game_state: GameState,
        promoting_move: Move,
    ) {
        let expected_color_of_promoted_figure = game_state.turn_by;
        let expected_promo_figure_type = if let Some(promoted_to) = promoting_move.promotion_type {
            promoted_to.get_figure_type()
        } else {
            panic!("expected move that includes a pawn promotion, but got {}", promoting_move)
        };
        let (new_game_state, _) = game_state.do_move(promoting_move);
        let promoted_figure = new_game_state.board.get_figure(promoting_move.clone().from_to.to);
        if let Some(figure) = promoted_figure {
            println!("{}", new_game_state.get_fen_part1to4());
            assert_eq!(figure.color, expected_color_of_promoted_figure);
            assert_eq!(figure.fig_type, expected_promo_figure_type);
        } else {
            panic!("expected a figure on promotion square")
        }
    }

    #[rstest(
        game_state, castling_move, expected_updated_board_fen,
        case("white ♖a1 ♔e1 ♖h1 ♜a8 ♚e8 ♜h8", "e1a1", "r3k2r/8/8/8/8/8/8/2KR3R"),
        case("white ♖a1 ♔e1 ♖h1 ♜a8 ♚e8 ♜h8", "e1h1", "r3k2r/8/8/8/8/8/8/R4RK1"),
        case("black ♖a1 ♔e1 ♖h1 ♜a8 ♚e8 ♜h8", "e8a8", "2kr3r/8/8/8/8/8/8/R3K2R"),
        case("black ♖a1 ♔e1 ♖h1 ♜a8 ♚e8 ♜h8", "e8h8", "r4rk1/8/8/8/8/8/8/R3K2R"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_castling_works(
        game_state: GameState,
        castling_move: Move,
        expected_updated_board_fen: &str,
    ) {
        let (new_game_state, _) = game_state.do_move(castling_move);
        let actual_updated_board_fen = new_game_state.board.get_fen_part1();
        assert_eq!(actual_updated_board_fen, expected_updated_board_fen);
    }

    #[rstest(
        game_state, expected_moves_played,
        case("", ""),
        case("e2e4", "e2e4"),
        case("e2e4 e7e5 g1f3", "e2e4, e7e5, g1f3"),
        ::trace //This leads to the arguments being printed in front of the test result.
    )]
    fn test_get_moves_played(
        game_state: GameState,
        expected_moves_played: &str,
    ) {
        let actual_moves_played: String = game_state.get_moves_played().iter().map(|it|format!("{it}")).join(", ");
        assert_eq!(actual_moves_played, expected_moves_played.to_string(), "moves played");
    }
}
