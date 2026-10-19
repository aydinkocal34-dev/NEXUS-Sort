//! Kristal Magarasi bolumu: elle tasarlanmis 8x8 tahta ve 500 puan hedefi.

use crate::game::board::{Board, BoardError};
use std::error::Error;
use std::fmt;

pub const TARGET_SCORE: u64 = 500;
pub const POINTS_PER_CRYSTAL: u64 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrystalCavernError {
    Board(BoardError),
    ScoreOverflow,
}
impl fmt::Display for CrystalCavernError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Board(error) => write!(f, "Tahta hatasi: {error}"),
            Self::ScoreOverflow => write!(f, "Skor siniri asildi."),
        }
    }
}
impl Error for CrystalCavernError {}
impl From<BoardError> for CrystalCavernError {
    fn from(error: BoardError) -> Self { Self::Board(error) }
}

#[derive(Debug, Clone)]
pub struct CrystalCavernLevel {
    board: Board,
    score: u64,
    moves: u64,
    cleared_crystals: u64,
}
impl CrystalCavernLevel {
    pub fn new() -> Result<Self, CrystalCavernError> {
        let rows = [
            "RGBYRGBY",
            "GBYRGBYR",
            "BYRGBYRG",
            "YRGBYRGB",
            "RBGYRBGY",
            "GYBRGYBR",
            "BRYG BRYG".replace(' ', ""),
            "YGRBYGRB",
        ];
        let row_refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        let board = Board::from_symbols(&row_refs, 0xC0A57A1)?;
        Ok(Self { board, score: 0, moves: 0, cleared_crystals: 0 })
    }

    pub fn board(&self) -> &Board { &self.board }
    pub fn score(&self) -> u64 { self.score }
    pub fn moves(&self) -> u64 { self.moves }
    pub fn cleared_crystals(&self) -> u64 { self.cleared_crystals }
    pub fn remaining_target(&self) -> u64 { TARGET_SCORE.saturating_sub(self.score) }
    pub fn is_complete(&self) -> bool { self.score >= TARGET_SCORE }
    pub fn victory_message(&self) -> String {
        if self.is_complete() {
            format!("Kristal Magarasi Aydinlandi! Skor: {} / {}", self.score, TARGET_SCORE)
        } else {
            format!("Kalan hedef: {} puan", self.remaining_target())
        }
    }

    pub fn play_at(&mut self, row: usize, col: usize) -> Result<usize, CrystalCavernError> {
        if self.is_complete() { return Ok(0); }
        let group_size = self.board.group_size(row, col)?;
        if group_size < 3 { return Ok(0); }
        let count = u64::try_from(group_size).map_err(|_| CrystalCavernError::ScoreOverflow)?;
        let gained = count.checked_mul(POINTS_PER_CRYSTAL).ok_or(CrystalCavernError::ScoreOverflow)?;
        let new_score = self.score.checked_add(gained).ok_or(CrystalCavernError::ScoreOverflow)?;
        let new_cleared = self.cleared_crystals.checked_add(count).ok_or(CrystalCavernError::ScoreOverflow)?;
        let new_moves = self.moves.checked_add(1).ok_or(CrystalCavernError::ScoreOverflow)?;
        let cleared = self.board.clear_group(row, col)?;
        if cleared != group_size {
            return Err(CrystalCavernError::Board(BoardError::InvalidRows));
        }
        self.score = new_score;
        self.cleared_crystals = new_cleared;
        self.moves = new_moves;
        Ok(cleared)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero_and_has_target() {
        let level = CrystalCavernLevel::new();
        assert!(level.is_ok());
        if let Ok(level) = level {
            assert_eq!(level.score(), 0);
            assert_eq!(level.remaining_target(), TARGET_SCORE);
            assert!(!level.is_complete());
            assert_eq!(level.board().size(), 8);
        }
    }

    #[test]
    fn board_is_filled_and_initially_has_no_triples() {
        let level = CrystalCavernLevel::new();
        assert!(level.is_ok());
        if let Ok(level) = level {
            for row in 0..8 {
                for col in 0..8 {
                    assert!(matches!(level.board().cell(row, col), Ok(Some(_))));
                    if col + 2 < 8 {
                        let a = level.board().cell(row, col);
                        let b = level.board().cell(row, col + 1);
                        let c = level.board().cell(row, col + 2);
                        assert!(a.is_ok() && b.is_ok() && c.is_ok());
                        assert!(a.ok() != b.ok() || b.ok() != c.ok());
                    }
                    if row + 2 < 8 {
                        let a = level.board().cell(row, col);
                        let b = level.board().cell(row + 1, col);
                        let c = level.board().cell(row + 2, col);
                        assert!(a.is_ok() && b.is_ok() && c.is_ok());
                        assert!(a.ok() != b.ok() || b.ok() != c.ok());
                    }
                }
            }
        }
    }

    #[test]
    fn scoring_and_victory_threshold_work() {
        assert_eq!(3_u64 * POINTS_PER_CRYSTAL, 30);
        let level = CrystalCavernLevel::new();
        assert!(level.is_ok());
        if let Ok(mut level) = level {
            // Use an explicitly prepared test board with a removable group.
            let board = Board::from_symbols(&["RRRRGBBY", "GBYRGBYR", "BYRGBYRG", "YRGBYRGB",
                "RBGYRBGY", "GYBRGYBR", "BRYGBRYG", "YGRBYGRB"], 4);
            assert!(board.is_ok());
            if let Ok(board) = board {
                level.board = board;
                assert_eq!(level.play_at(0, 0), Ok(4));
                assert_eq!(level.score(), 40);
                assert_eq!(level.remaining_target(), 460);
            }
        }
    }

    #[test]
    fn invalid_coordinates_return_error() {
        let level = CrystalCavernLevel::new();
        assert!(level.is_ok());
        if let Ok(mut level) = level {
            assert!(level.play_at(8, 0).is_err());
        }
    }
}
