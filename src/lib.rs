use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PieceColor {
    White,
    Black,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Piece {
    piece_type: PieceType,
    piece_color: PieceColor,
}

impl Piece {
    fn new(piece_type: PieceType, piece_color: PieceColor) -> Self {
        Piece {
            piece_type,
            piece_color,
        }
    }
}

#[derive(Debug)]
pub struct Board {
    square: [Option<Piece>; 64],
    current_turn: PieceColor,
}

#[derive(Debug)]
pub struct MoveList {
    pub moves: [usize; 27],
    pub count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [0; 27],
            count: 0,
        }
    }

    pub fn push(&mut self, idx: usize) {
        self.moves[self.count] = idx;
        self.count += 1;
    }
}

impl Board {
    pub fn new() -> Self {
        let mut square: [Option<Piece>; 64] = [None; 64];

        for i in 0..64 {
            let (row, col) = Board::index_to_coordinates(i);

            let piece_color = if row < 2 {
                PieceColor::White
            } else if row > 5 {
                PieceColor::Black
            } else {
                continue;
            };

            let piece_type = match row {
                0 | 7 => match col {
                    0 | 7 => PieceType::Rook,
                    1 | 6 => PieceType::Knight,
                    2 | 5 => PieceType::Bishop,
                    3 => PieceType::Queen,
                    4 => PieceType::King,
                    _ => unreachable!(),
                },
                1 | 6 => PieceType::Pawn,
                _ => unreachable!(),
            };

            square[i] = Some(Piece::new(piece_type, piece_color));
        }

        Board {
            square,
            current_turn: PieceColor::White,
        }
    }

    fn index_to_coordinates(index: usize) -> (usize, usize) {
        let row = index / 8;
        let col = index % 8;
        (row, col)
    }

    pub fn move_piece(&mut self, current_idx: usize, target_idx: usize) -> Result<(), &'static str> {
        let possible_moves = self.get_possible_moves(current_idx)?;

        let valid_moves = &possible_moves.moves[0..possible_moves.count];
        if !valid_moves.contains(&target_idx) {
            return Err("Invalid move!");
        }

        self.square[target_idx] = self.square[current_idx].take();
        self.current_turn = match self.current_turn {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
        Ok(())
    }

    pub fn get_possible_moves(&self, idx: usize) -> Result<MoveList, &'static str> {
        let Some(piece) = self.square[idx] else {
            return Err("No piece at the given index");
        };
        if piece.piece_color != self.current_turn {
            return Err("It's not the current player's turn");
        }
        match piece.piece_type {
            PieceType::Pawn => Ok(self.get_possible_pawn_moves(piece, idx)),
            PieceType::Knight => Ok(self.get_possible_knight_moves(piece, idx)),
            PieceType::Rook => Ok(self.get_possible_rook_moves(piece, idx)),
            PieceType::Bishop => Ok(self.get_possible_bishop_moves(piece, idx)),
            PieceType::Queen => Ok(self.get_possible_queen_moves(piece, idx)),
            PieceType::King => Ok(self.get_possible_king_moves(piece, idx)),
        }
    }

    fn get_possible_pawn_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let r = row as isize;
        let c = col as isize;
        let row_step: isize = if piece.piece_color == PieceColor::White { 1 } else { -1 };

        // Check forward
        for j in [1, 2] {
            let row_step = j*row_step;
            if (r + row_step) > 7 || (r + row_step) < 0 {
                break;
            }
            let target_idx = 8*((r+row_step) as usize)+col;
            let None = self.square[target_idx] else { break; };
            if j==2 && !((row == 1 && piece.piece_color == PieceColor::White) || (row == 6 && piece.piece_color == PieceColor::Black)) {
                break;
            }
            result.push(target_idx)
        }

        // Check Diagonally
        for col_step in [1, -1] {
            if (r + row_step) >= 0 && (r + row_step) <= 7 && (c + col_step) >= 0 && (c + col_step) <= 7 {
                let target_idx = (8 * (r+row_step) + (c + col_step)) as usize;
                if let Some(target_piece) = self.square[target_idx] {
                    if target_piece.piece_color != piece.piece_color {
                        result.push(target_idx);
                    }
                }
            }
        }

        result
    }

    fn get_possible_knight_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)];
        self.get_directional_moves(&mut result, piece, row, col, &directions, 1);
        result
    }

    fn get_possible_rook_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        self.get_directional_moves(&mut result, piece, row, col, &directions, 7);
        result
    }

    fn get_possible_bishop_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1,1), (-1,1), (1,-1), (-1,-1)];
        self.get_directional_moves(&mut result, piece, row, col, &directions, 7);
        result
    }

    fn get_possible_queen_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1,1), (-1,1), (1,-1), (-1,-1), (1, 0), (-1, 0), (0, 1), (0, -1)];
        self.get_directional_moves(&mut result, piece, row, col, &directions, 7);
        result
    }

    fn get_possible_king_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1,1), (-1,1), (1,-1), (-1,-1), (1, 0), (-1, 0), (0, 1), (0, -1)];
        self.get_directional_moves(&mut result, piece, row, col, &directions, 1);
        result
    }

    fn get_directional_moves (&self, result: &mut MoveList, piece: Piece, row: usize, col: usize, directions: &[(isize, isize)], max_steps: usize) {
        let r = row as isize;
        let c = col as isize;

        for &(row_step, col_step) in directions {
            let mut current_r = r + row_step;
            let mut current_c = c + col_step;

            let mut steps: usize = 0;
            while current_r <= 7 && current_r >= 0 && current_c <= 7 && current_c >= 0 {
                let target_idx = (8 * current_r + current_c) as usize;

                match self.square[target_idx] {
                    Some(target_piece) => {
                        if target_piece.piece_color != piece.piece_color {
                            result.push(target_idx);
                        }
                        break;
                    }
                    None => {
                        result.push(target_idx);
                        steps += 1;
                        if steps >= max_steps { break; }
                        current_r += row_step;
                        current_c += col_step;
                    }
                }
            }
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         for i in (0..8).rev() {
            for j in 0..8 {
                let idx = 8 * i + j;
                let Some(piece) = self.square[idx].as_ref() else {
                    write!(f, "o ")?;
                    continue;
                };
                let char = match piece.piece_type {
                            PieceType::Pawn => 'p',
                            PieceType::Knight => 'n',
                            PieceType::Bishop => 'b',
                            PieceType::Rook => 'r',
                            PieceType::Queen => 'q',
                            PieceType::King => 'k',
                };
                let display_char = match piece.piece_color {
                    PieceColor::White => char.to_ascii_uppercase(),
                    _ => char
                };
                write!(f, "{} ", display_char)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

