use std::str::Chars;
use crate::base::a_move::{FromTo, Move, MoveData, PromotionType};
use crate::base::errors::{ChessError, ErrorKind};
use crate::base::position::Position;
use crate::compression::base64::{assert_is_url_safe_base64, decode_base64};
use crate::figure::functions::is_reachable_by::get_positions_to_reach_target_from;
use crate::game::game_state::GameState;

/// the length of Vec<PositionData> is 1 higher than the length of Vec<MoveData>, since the initial Position exists before the first move
pub fn decompress(base64_encoded_match: &str) -> Result<(Vec<PositionData>, Vec<MoveData>), ChessError> {
    assert_is_url_safe_base64(base64_encoded_match)?;

    fn get_next_position(encoded_chars: &mut Chars) -> Result<Option<Position>, ChessError> {
        match encoded_chars.next() {
            None => { Ok(None) }
            Some(base64_char) => {
                let position = decode_base64(base64_char)?;
                Ok(Some(position))
            }
        }
    }

    let mut encoded_chars: Chars = base64_encoded_match.chars();
    let mut game_state = GameState::classic();
    let mut moves_played: Vec<MoveData> = Vec::new();
    let mut positions_reached: Vec<PositionData> = {
        let mut positions_data = Vec::new();
        positions_data.push(PositionData::new(game_state.get_fen()));
        positions_data
    };

    let mut half_move_index = 0;
    loop {
        let move_index = half_move_index / 2;

        let next_move = {
            let active_color = game_state.turn_by;
            let first_pos: Position = match get_next_position(&mut encoded_chars)? {
                None => { break; }
                Some(pos) => { pos }
            };

            let from_to = if game_state.board.contains_color(first_pos, active_color) {
                let to_pos: Position = match get_next_position(&mut encoded_chars)? {
                    None => {
                        return Err(ChessError {
                            msg: format!("second position missing for {move_index} move for {active_color} after start position was {first_pos}"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                    Some(pos) => { pos }
                };
                FromTo::new(first_pos, to_pos)
            } else {
                let positions_with_figures_that_can_reach_target: Vec<Position> = get_positions_to_reach_target_from(first_pos, &game_state)?;
                let from_to: FromTo = match positions_with_figures_that_can_reach_target.len() {
                    0 => {
                        return Err(ChessError {
                            msg: format!("no position found that could reach {first_pos} in move {move_index} for {active_color}"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                    1 => { FromTo::new(positions_with_figures_that_can_reach_target[0], first_pos) }
                    _ => {
                        return Err(ChessError {
                            msg: format!("many position found that could reach {move_index} in move {active_color} for {first_pos}: {positions_with_figures_that_can_reach_target:?}"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                };
                from_to
            };

            if game_state.looks_like_pawn_promotion_move(from_to) {
                let promotion_type: PromotionType = match encoded_chars.next() {
                    None => {
                        return Err(ChessError {
                            msg: format!("missing pawn promotion type at last decoded move {from_to}, one of 'Q', 'R', 'N' or 'B' was expected next depending on what figure the pawn should promoted to"),
                            kind: ErrorKind::IllegalFormat,
                        });
                    }
                    Some(promotion_type_char) => {
                        match promotion_type_char.to_string().parse::<PromotionType>()  {
                            Ok(promotion_type) => {promotion_type}
                            Err(_) => {
                                return Err(ChessError {
                                    msg: format!("missing pawn promotion at decoded move {move_index}. {from_to}, one of 'Q', 'R', 'N' or 'B' was expected next depending on what figure the pawn should promoted to"),
                                    kind: ErrorKind::IllegalFormat,
                                });
                            }
                        }
                    }
                };
                Move::new_with_promotion(from_to, promotion_type)
            } else {
                Move::new(from_to)
            }
        };

        let (new_game_state, latest_move_data) = game_state.do_move(next_move);
        game_state = new_game_state;
        positions_reached.push(PositionData::new(game_state.get_fen()));
        moves_played.push(latest_move_data);
        half_move_index = half_move_index + 1;
    }

    Ok((positions_reached, moves_played))
}

pub struct PositionData {
    pub fen: String,
}

impl PositionData {
    pub fn new(fen: String) -> PositionData {
        PositionData {
            fen,
        }
    }
}

// Tests are in compression/mod.rs
