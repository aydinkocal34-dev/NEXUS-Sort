pub mod board;
pub mod level;
pub mod lives;

use board::{Board, BoardError};
use level::{create_board, LevelDefinition};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError { Board(BoardError), ScoreOverflow, BoardStateChanged }
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Board(e) => write!(f, "Board error: {e}"),
            Self::ScoreOverflow => write!(f, "Score counter overflow."),
            Self::BoardStateChanged => write!(f, "Board changed unexpectedly during a move."),
        }
    }
}
impl Error for GameError {}
impl From<BoardError> for GameError {
    fn from(error: BoardError) -> Self { Self::Board(error) }
}

#[derive(Debug, Clone)]
pub struct GameState {
    board: Board,
    score: u64,
    level_name: &'static str,
    cleared_pieces: u64,
    moves: u64,
}
impl GameState {
    pub fn new(level: &LevelDefinition) -> Result<Self, GameError> {
        Ok(Self { board: create_board(level)?, score: 0, level_name: level.name, cleared_pieces: 0, moves: 0 })
    }
    pub fn board(&self) -> &Board { &self.board }
    pub fn score(&self) -> u64 { self.score }
    pub fn level_name(&self) -> &'static str { self.level_name }
    pub fn cleared_pieces(&self) -> u64 { self.cleared_pieces }
    pub fn moves(&self) -> u64 { self.moves }

    pub fn play_at(&mut self, row: usize, col: usize) -> Result<usize, GameError> {
        let group_size = self.board.group_size(row, col)?;
        if group_size < 3 { return Ok(0); }
        let count = u64::try_from(group_size).map_err(|_| GameError::ScoreOverflow)?;
        let gained = count.checked_mul(10).ok_or(GameError::ScoreOverflow)?;
        let new_score = self.score.checked_add(gained).ok_or(GameError::ScoreOverflow)?;
        let new_cleared = self.cleared_pieces.checked_add(count).ok_or(GameError::ScoreOverflow)?;
        let new_moves = self.moves.checked_add(1).ok_or(GameError::ScoreOverflow)?;
        let cleared = self.board.clear_group(row, col)?;
        if cleared != group_size { return Err(GameError::BoardStateChanged); }
        self.score = new_score;
        self.cleared_pieces = new_cleared;
        self.moves = new_moves;
        Ok(cleared)
    }

    #[cfg(test)]
    fn set_score_for_test(&mut self, score: u64) { self.score = score; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::level::LEVELS;
    #[test] fn starts_with_zero_score() {
        let game = GameState::new(&LEVELS[0]);
        assert!(game.is_ok());
        if let Ok(game) = game { assert_eq!(game.score(), 0); assert_eq!(game.moves(), 0); }
    }
    #[test] fn successful_clear_updates_score() {
        let level = LevelDefinition { name: "Test", description: "test", rows: &["RRB", "RGB", "YGB"], seed: 1 };
        let game = GameState::new(&level);
        assert!(game.is_ok());
        if let Ok(mut game) = game {
            assert_eq!(game.play_at(0, 0), Ok(3));
            assert_eq!(game.score(), 30);
            assert_eq!(game.cleared_pieces(), 3);
            assert_eq!(game.moves(), 1);
        }
    }
    #[test] fn invalid_selection_does_not_score() {
        let level = LevelDefinition { name: "Test", description: "test", rows: &["RGB", "YPR", "GBY"], seed: 1 };
        let game = GameState::new(&level);
        assert!(game.is_ok());
        if let Ok(mut game) = game {
            assert_eq!(game.play_at(0, 0), Ok(0));
            assert_eq!(game.score(), 0);
            assert_eq!(game.moves(), 0);
        }
    }
    #[test] fn overflow_does_not_change_board_or_counters() {
        let level = LevelDefinition { name: "Test", description: "test", rows: &["RRB", "RGB", "YGB"], seed: 1 };
        let game = GameState::new(&level);
        assert!(game.is_ok());
        if let Ok(mut game) = game {
            game.set_score_for_test(u64::MAX - 5);
            let before = game.board().render();
            assert!(matches!(game.play_at(0, 0), Err(GameError::ScoreOverflow)));
            assert_eq!(game.board().render(), before);
            assert_eq!(game.score(), u64::MAX - 5);
            assert_eq!(game.moves(), 0);
        }
    }
}

pub mod crystal_cavern;
