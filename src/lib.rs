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

