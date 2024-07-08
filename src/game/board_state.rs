use tinyvec::*;
use crate::game::{StoppedReason, MoveData};
use crate::base::Color;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct BoardState {
    state: [u64; 4],
}

impl BoardState {
    pub fn new(state: [u64; 4]) -> BoardState {
        BoardState { state }
    }
}

// Default is needed, so that BoardState can be stored in a TinyVec
// impl Default for BoardState {
//     fn default() -> Self {
//         BoardState {
//             state: [0;4],
//         }
//     }
// }

const IN_MEMORY_NR_OF_BOARD_STATES: usize = 20;

#[derive(Clone)]
pub struct BoardStateArray {
    array: [BoardState; IN_MEMORY_NR_OF_BOARD_STATES]
}

impl tinyvec::Array for BoardStateArray {
    type Item = BoardState;
    const CAPACITY: usize = IN_MEMORY_NR_OF_BOARD_STATES;

    fn as_slice(&self) -> &[Self::Item] {
        &self.array
    }

    fn as_slice_mut(&mut self) -> &mut [Self::Item] {
        &mut self.array
    }

    fn default() -> Self {
        BoardStateArray {
            array: [BoardState::default(); IN_MEMORY_NR_OF_BOARD_STATES]
        }
    }
}

type OneSidedBoardStates = TinyVec<BoardStateArray>;

#[derive(Debug, Clone)]
pub struct BoardStates {
    white_board_states_history: OneSidedBoardStates,
    black_board_states_history: OneSidedBoardStates,
}

impl BoardStates {
    pub fn new(current_board_state: BoardState, turn_by: Color) -> BoardStates {
        let mut past_white_board_states: OneSidedBoardStates = tiny_vec!();
        let mut past_black_board_states: OneSidedBoardStates = tiny_vec!();
        match turn_by {
            Color::White => {past_white_board_states.push(current_board_state)}
            Color::Black => {past_black_board_states.push(current_board_state)}
        };
        BoardStates {
            white_board_states_history: past_white_board_states,
            black_board_states_history: past_black_board_states,
        }
    }

    pub fn add_board_state_and_check_for_draw(&self, new_board_state: BoardState, turn_by: Color, a_move: MoveData) -> Result<BoardStates, StoppedReason> {
        let (
            new_white_board_states_history,
            new_black_board_states_history,
        ) = {
            if a_move.is_pawn_move() || a_move.did_catch_figure() {
                let mut new_white_board_states = tiny_vec!();
                let mut new_black_board_states = tiny_vec!();
                match turn_by {
                    Color::White => { new_white_board_states.push(new_board_state); }
                    Color::Black => { new_black_board_states.push(new_board_state); }
                };
                (new_white_board_states, new_black_board_states)
            } else {
                let mut new_white_board_states = self.white_board_states_history.clone();
                let mut new_black_board_states = self.black_board_states_history.clone();

                fn push_onto_list_and_checks_if_three_fold_repetition(
                    board_state: BoardState,
                    board_states: &mut OneSidedBoardStates,
                ) -> bool {

                    let is_three_fold_repetition: bool = if board_states.len() < 4 {
                        false
                    } else {
                        let mut occurrence_of_board_state: usize = 1;
                        board_states.iter().for_each(|it| {
                            if board_state.eq(it) {
                                occurrence_of_board_state += 1;
                            }
                        });
                        debug_assert!(
                            occurrence_of_board_state<4,
                            "maximum occurrence of a board state should be 3 but is {}",
                            occurrence_of_board_state
                        );
                        occurrence_of_board_state == 3
                    };
                    board_states.push(board_state);
                    is_three_fold_repetition
                }

                let is_three_fold_repetition = match turn_by {
                    Color::White => {
                        push_onto_list_and_checks_if_three_fold_repetition(
                            new_board_state,
                            &mut new_white_board_states,
                        )
                    }
                    Color::Black => {
                        push_onto_list_and_checks_if_three_fold_repetition(
                            new_board_state,
                            &mut new_black_board_states,
                        )
                    }
                };
                if is_three_fold_repetition {
                    return Err(StoppedReason::ThreeTimesRepetition);
                }
                (new_white_board_states, new_black_board_states)
            }
        };

        debug_assert!(
            {
                let white_states_count = new_white_board_states_history.len() as isize;
                let black_states_count = new_black_board_states_history.len() as isize;
                (white_states_count-black_states_count).abs() <2
            },
            "number of white({})/black({}) board_states can only differ by 1",
            new_white_board_states_history.len(),
            new_black_board_states_history.len(),
        );
        debug_assert!(
            {
                if a_move.did_catch_figure() || a_move.is_pawn_move() {
                    let white_states_count = new_white_board_states_history.len() as isize;
                    let black_states_count = new_black_board_states_history.len() as isize;
                    match turn_by {
                        Color::White => {
                            white_states_count == 1 && black_states_count == 0
                        }
                        Color::Black => {
                            white_states_count == 0 && black_states_count == 1
                        }
                    }
                } else {true}
            },
            "catching a figure or moving a pawn resets the board states: color {}, white# {}, black# {}",
            turn_by,
            new_white_board_states_history.len(),
            new_black_board_states_history.len(),
        );

        if new_white_board_states_history.len() == 50 && new_black_board_states_history.len() == 50 {
            Err(StoppedReason::NoChangeIn50Moves)
        } else {
            Ok(BoardStates {
                white_board_states_history: new_white_board_states_history,
                black_board_states_history: new_black_board_states_history,
            })
        }
    }

    pub fn count_half_moves_without_progress(&self) -> usize {
        self.white_board_states_history.len() + self.black_board_states_history.len() - 1
    }
}
