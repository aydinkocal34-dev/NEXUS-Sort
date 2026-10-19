use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color { Red, Green, Blue, Yellow, Purple }

impl Color {
    pub fn symbol(self) -> char {
        match self {
            Self::Red => 'R', Self::Green => 'G', Self::Blue => 'B',
            Self::Yellow => 'Y', Self::Purple => 'P',
        }
    }
    fn from_symbol(c: char) -> Option<Self> {
        match c {
            'R' => Some(Self::Red), 'G' => Some(Self::Green),
            'B' => Some(Self::Blue), 'Y' => Some(Self::Yellow),
            'P' => Some(Self::Purple), _ => None,
        }
    }
    fn from_index(i: u64) -> Self {
        match i % 5 {
            0 => Self::Red, 1 => Self::Green, 2 => Self::Blue,
            3 => Self::Yellow, _ => Self::Purple,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardError {
    InvalidSize,
    InvalidRows,
    InvalidSymbol(char),
    OutOfBounds { row: usize, col: usize },
}
impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize => write!(f, "Grid size must be at least 3."),
            Self::InvalidRows => write!(f, "Grid must be square with equal row lengths."),
            Self::InvalidSymbol(c) => write!(f, "Invalid piece symbol: '{c}'."),
            Self::OutOfBounds { row, col } => write!(f, "Grid coordinate out of bounds: ({row}, {col})."),
        }
    }
}
impl Error for BoardError {}

#[derive(Debug, Clone)]
pub struct Board {
    size: usize,
    cells: Vec<Option<Color>>,
    rng_state: u64,
}
impl Board {
    pub fn new(size: usize, seed: u64) -> Result<Self, BoardError> {
        if size < 3 { return Err(BoardError::InvalidSize); }
        let count = size.checked_mul(size).ok_or(BoardError::InvalidSize)?;
        let mut board = Self { size, cells: vec![None; count], rng_state: seed };
        for i in 0..count {
            let color = Color::from_index(board.next_random());
            board.cells[i] = Some(color);
        }
        Ok(board)
    }

    pub fn from_symbols(rows: &[&str], seed: u64) -> Result<Self, BoardError> {
        let size = rows.len();
        if size < 3 { return Err(BoardError::InvalidSize); }
        let count = size.checked_mul(size).ok_or(BoardError::InvalidSize)?;
        let mut cells = Vec::with_capacity(count);
        for row in rows {
            let symbols: Vec<char> = row.chars().collect();
            if symbols.len() != size { return Err(BoardError::InvalidRows); }
            for c in symbols {
                cells.push(if c == '.' { None } else {
                    Some(Color::from_symbol(c).ok_or(BoardError::InvalidSymbol(c))?)
                });
            }
        }
        Ok(Self { size, cells, rng_state: seed })
    }

    pub fn size(&self) -> usize { self.size }
    fn index(&self, row: usize, col: usize) -> Result<usize, BoardError> {
        if row >= self.size || col >= self.size {
            return Err(BoardError::OutOfBounds { row, col });
        }
        Ok(row * self.size + col)
    }
    pub fn cell(&self, row: usize, col: usize) -> Result<Option<Color>, BoardError> {
        Ok(self.cells[self.index(row, col)?])
    }
    fn next_random(&mut self) -> u64 {
        self.rng_state = self.rng_state.wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.rng_state >> 32
    }
    fn random_color(&mut self) -> Color { Color::from_index(self.next_random()) }
    fn neighbors(&self, index: usize) -> Vec<usize> {
        let row = index / self.size;
        let col = index % self.size;
        let mut out = Vec::with_capacity(4);
        if row > 0 { out.push(index - self.size); }
        if row + 1 < self.size { out.push(index + self.size); }
        if col > 0 { out.push(index - 1); }
        if col + 1 < self.size { out.push(index + 1); }
        out
    }
    fn group_indices(&self, start: usize) -> Vec<usize> {
        let Some(color) = self.cells[start] else { return Vec::new(); };
        let mut seen = vec![false; self.cells.len()];
        let mut queue = VecDeque::new();
        let mut group = Vec::new();
        seen[start] = true;
        queue.push_back(start);
        while let Some(current) = queue.pop_front() {
            group.push(current);
            for next in self.neighbors(current) {
                if !seen[next] && self.cells[next] == Some(color) {
                    seen[next] = true;
                    queue.push_back(next);
                }
            }
        }
        group
    }
    pub fn group_size(&self, row: usize, col: usize) -> Result<usize, BoardError> {
        Ok(self.group_indices(self.index(row, col)?).len())
    }
    pub fn clear_group(&mut self, row: usize, col: usize) -> Result<usize, BoardError> {
        let group = self.group_indices(self.index(row, col)?);
        if group.len() < 3 { return Ok(0); }
        let cleared = group.len();
        for index in group { self.cells[index] = None; }
        self.apply_gravity_and_refill();
        Ok(cleared)
    }
    fn apply_gravity_and_refill(&mut self) {
        for col in 0..self.size {
            let mut write_row = self.size;
            for read_row in (0..self.size).rev() {
                let read_index = read_row * self.size + col;
                if let Some(color) = self.cells[read_index] {
                    write_row -= 1;
                    let write_index = write_row * self.size + col;
                    self.cells[write_index] = Some(color);
                }
            }
            for row in 0..write_row {
                let color = self.random_color();
                self.cells[row * self.size + col] = Some(color);
            }
        }
    }
    pub fn render(&self) -> String {
        let mut out = String::from("    ");
        for col in 0..self.size { out.push_str(&format!("{:2} ", col + 1)); }
        out.push('\n');
        for row in 0..self.size {
            out.push_str(&format!("{:3} ", row + 1));
            for col in 0..self.size {
                let symbol = match self.cells[row * self.size + col] {
                    Some(c) => c.symbol(), None => '.',
                };
                out.push_str(&format!(" {} ", symbol));
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_small_board() { assert!(matches!(Board::new(2, 1), Err(BoardError::InvalidSize))); }
    #[test] fn finds_connected_groups() {
        let board = Board::from_symbols(&["RRG", "RBG", "BBB"], 1);
        assert!(board.is_ok());
        if let Ok(board) = board {
            assert_eq!(board.group_size(0, 0), Ok(3));
            assert_eq!(board.group_size(2, 1), Ok(4));
        }
    }
    #[test] fn groups_smaller_than_three_are_not_cleared() {
        let board = Board::from_symbols(&["RGB", "YPR", "GBY"], 1);
        assert!(board.is_ok());
        if let Ok(mut board) = board {
            let before = board.render();
            assert_eq!(board.clear_group(0, 0), Ok(0));
            assert_eq!(board.render(), before);
        }
    }
    #[test] fn clear_refills_all_cells() {
        let board = Board::from_symbols(&["RRB", "RGB", "YGB"], 123);
        assert!(board.is_ok());
        if let Ok(mut board) = board {
            assert_eq!(board.clear_group(0, 0), Ok(3));
            for row in 0..board.size() {
                for col in 0..board.size() {
                    assert!(matches!(board.cell(row, col), Ok(Some(_))));
                }
            }
        }
    }
    #[test] fn rejects_invalid_coordinates_and_symbols() {
        let board = Board::new(3, 1);
        assert!(board.is_ok());
        if let Ok(board) = board {
            assert!(matches!(board.cell(3, 0), Err(BoardError::OutOfBounds { .. })));
        }
        assert!(matches!(Board::from_symbols(&["RRX", "RGB", "YGB"], 1), Err(BoardError::InvalidSymbol('X'))));
    }
    #[test] fn seeded_board_is_deterministic() {
        let a = Board::new(5, 99);
        let b = Board::new(5, 99);
        assert!(a.is_ok() && b.is_ok());
        if let (Ok(a), Ok(b)) = (a, b) { assert_eq!(a.render(), b.render()); }
    }
}
