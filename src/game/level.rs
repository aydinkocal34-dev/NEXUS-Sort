use crate::game::board::{Board, BoardError};

#[derive(Debug, Clone, Copy)]
pub struct LevelDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub rows: &'static [&'static str],
    pub seed: u64,
}

pub const LEVELS: [LevelDefinition; 3] = [
    LevelDefinition { name: "Kolay", description: "Buyuk renk gruplarini bul.", rows: &["RRGGB", "RYYGB", "PPYGB", "PPRRB", "GGYRB"], seed: 101 },
    LevelDefinition { name: "Orta", description: "Kucuk gruplari dikkatle sec.", rows: &["RGBYR", "RGBYP", "YGBPR", "YGBPR", "PRYGB"], seed: 202 },
    LevelDefinition { name: "Zor", description: "Baglantilari planlayarak puan topla.", rows: &["RGPYR", "RGPYR", "BYRGP", "BYRGP", "YPRGB"], seed: 303 },
];

pub fn get_level(index: usize) -> Option<&'static LevelDefinition> { LEVELS.get(index) }
pub fn create_board(level: &LevelDefinition) -> Result<Board, BoardError> {
    Board::from_symbols(level.rows, level.seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn has_three_levels() { assert_eq!(LEVELS.len(), 3); }
    #[test] fn each_level_creates_full_board() {
        for level in &LEVELS {
            let board = create_board(level);
            assert!(board.is_ok());
            if let Ok(board) = board {
                assert_eq!(board.size(), 5);
                for r in 0..5 { for c in 0..5 { assert!(matches!(board.cell(r, c), Ok(Some(_)))); } }
            }
        }
    }
    #[test] fn rejects_unknown_level() { assert!(get_level(3).is_none()); }
}
