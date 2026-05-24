//! # TTY Mate Chess Engine
//! This library provides the core board and essensial functions for the tty mate chess engine. It
//! defines the data structures for representing the chess board, pieces, and game state, as well
//! as the logic for move generation, move validation, and game rules enforcement.

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub  enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceColor {
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
    checked: bool,
    checkers: [Option<usize>; 2], // Max 2 pieces can check the king at once
    black_king_pos: usize,
    white_king_pos: usize,
    game_over: bool,
    winner: Option<PieceColor>,
    castle_rights: CastleRights,
    en_passant_target: Option<usize>, // To know if em passant is valid and keep track of the target piece
}

#[derive(Debug)]
struct CastleRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

// This structure is implemented to only iterate till possible moves
#[derive(Debug)]
pub struct MoveList {
    pub moves: [usize; 27], // Max possible moves for a queen in the center of an empty board is 27
    pub count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [0; 27],
            count: 0,
        }
    }

    fn push(&mut self, idx: usize) {
        self.moves[self.count] = idx;
        self.count += 1;
    }
}

impl Board {
    /// Initializes a new standard chess board.
    ///
    /// Sets up the initial 8x8 grid mapped to a 64-element flat array.
    /// It automatically populates the pieces in their standard starting positions.
    pub fn new() -> Self {
        let mut square: [Option<Piece>; 64] = [None; 64];

        for i in 0..64 {
            let (row, col) = Board::index_to_coordinates(i);

            let piece_color = if row < 2 {
                PieceColor::White // White pieces starting from 0th index
            } else if row > 5 {
                PieceColor::Black // Black pieces ending at 63rd index
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
            checked: false,
            checkers: [None; 2],
            black_king_pos: 60,
            white_king_pos: 4,
            game_over: false,
            winner: None,
            castle_rights: CastleRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: true,
                black_queenside: true,
            },
            en_passant_target: None,
        }
    }

    /// Converts a 1D array index (0-63) into 2D board coordinates (row, col).
    ///
    /// The board is mapped such that index 0 is (0, 0) a1 and index 63 is (7, 7) h8.
    ///
    /// # Examples
    /// ```
    /// use tty_mate_core::Board;
    /// 
    /// assert_eq!(Board::index_to_coordinates(0), (0, 0));
    /// assert_eq!(Board::index_to_coordinates(8), (1, 0));
    /// assert_eq!(Board::index_to_coordinates(63), (7, 7));
    /// ```
    pub fn index_to_coordinates(index: usize) -> (usize, usize) {
        let row = index / 8;
        let col = index % 8;
        (row, col)
    }

    /// Returns the exact board index of the King currently in check, if any.
    ///
    /// This is primarily used by the rendering engine to visually highlight
    /// the threatened King.
    pub fn get_checked_index(&self) -> Option<usize> {
        if self.checked {
            match self.current_turn {  // If a king is checked it belongs to the color whose turn it is
                PieceColor::White => Some(self.white_king_pos),
                PieceColor::Black => Some(self.black_king_pos),
            }
        } else {
            None
        }
    }


    /// Attempts to apply a strictly legal move to the board state.
    ///
    /// This is the core function to move pieces on the board. It verifies
    /// turn order, checks move legality (including FIDE rules like castling 
    /// en passant, and check avoidance), and applies the move.
    ///
    /// It automatically handles side-effects such as turn switching, pawn promotion,
    /// and checkmate detection.
    ///
    /// Returns `Ok(())` if the move was successful, or an `Err` explaining the rule violation.
    ///
    /// # Examples
    /// ```
    /// use tty_mate_core::Board;
    ///
    /// let mut board = Board::new();
    ///
    /// // Move White's pawn forward two squares
    /// assert!(board.move_piece(12, 28).is_ok());
    ///
    /// // Attempt to move White's piece out of turn
    /// assert!(board.move_piece(11, 27).is_err());
    /// ```
    pub fn move_piece(&mut self, current_idx: usize, target_idx: usize) -> Result<(), &'static str> {
        if self.game_over {
            return Err("Game is already over");
        }
        let possible_moves = self.get_valid_moves(current_idx)?; // Also checks if there's a piece at current_idx
        let piece = self.square[current_idx].unwrap(); // Safe to unwrap since we just checked it

        if piece.piece_color != self.current_turn {
            return Err("It's not your turn");
        }

        let valid_moves = &possible_moves.moves[0..possible_moves.count];
        if !valid_moves.contains(&target_idx) {
            return Err("Invalid move!");
        }

        let mut new_en_passant_target: Option<usize> = None;

        self.square[target_idx] = self.square[current_idx].take();
        if piece.piece_type == PieceType::King {
            let distance = target_idx as isize - current_idx as isize;
            if distance == 2 { // If distance is +/-2, it is a casling attempt
                self.square[current_idx + 1] = self.square[current_idx + 3].take(); // Kingside
            } else if distance == -2 {
                self.square[current_idx - 1] = self.square[current_idx - 4].take(); // Queenside
            }

            // Take away castle rights and update king pos if the king moves
            match piece.piece_color {
                PieceColor::Black => {
                    self.castle_rights.black_kingside = false; 
                    self.castle_rights.black_queenside = false;
                    self.black_king_pos = target_idx;
                },
                PieceColor::White => {
                    self.castle_rights.white_kingside = false;
                    self.castle_rights.white_queenside = false;
                    self.white_king_pos = target_idx;
                }
            }
        } else if piece.piece_type == PieceType::Pawn  {
            if target_idx < 8 || target_idx > 55 { // Pawn Promotion
                self.square[target_idx] = Some(Piece::new(PieceType::Queen, piece.piece_color)); // Auto promote to queen for MVP
            }
            if (target_idx as isize - current_idx as isize).abs() == 16 { // Two step move, Possible En Passant
                new_en_passant_target = Some((current_idx + target_idx) / 2);
            }
            if let Some(en_passant_idx) = self.en_passant_target {
                if target_idx == en_passant_idx {
                    if piece.piece_color == PieceColor::White {
                        self.square[en_passant_idx - 8] = None; // Capture the black pawn
                    } else {
                        self.square[en_passant_idx + 8] = None; // Capture the white pawn
                    }
                }
            }
        } else if piece.piece_type == PieceType::Rook {
            // Take away castle rights if the rook moves
            match current_idx {
                0 => self.castle_rights.white_queenside = false, 
                7 => self.castle_rights.white_kingside = false,
                56 => self.castle_rights.black_queenside = false,
                63 => self.castle_rights.black_kingside = false,
                _ => {}
            }
        }

        self.current_turn = match self.current_turn {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };

        let checkers = self.is_king_in_check(self.current_turn);
        if checkers[0].is_none() {
            self.checked = false;
        } else {
            self.checked = true;
        }
        self.checkers = checkers;

        if self.check_game_over() {
            self.game_over = true;
            if self.checked {
                self.winner = Some(match self.current_turn {
                    PieceColor::White => PieceColor::Black,
                    PieceColor::Black => PieceColor::White,
                });
            }
        }

        self.en_passant_target = new_en_passant_target;

        Ok(())
    }

    fn is_king_in_check(&self, king_color: PieceColor) -> [Option<usize>; 2] {
        let mut result = [None; 2];
        let mut i: usize = 0;
        let king_pos = match king_color {
            PieceColor::White => self.white_king_pos,
            PieceColor::Black => self.black_king_pos,
        };
        let king_piece = match king_color {
            PieceColor::White => Piece::new(PieceType::King, PieceColor::White),
            PieceColor::Black => Piece::new(PieceType::King, PieceColor::Black),
        };

        // Become a universal piece and cast rays in every possible direction, looking for possible
        // pieces in the way.
        let pawn_moves = self.get_possible_pawn_moves(king_piece, king_pos, true);
        if self.check_for_checks(pawn_moves, king_color, &[PieceType::Pawn], &mut result, &mut i) {
            return result;
        }
        let knight_moves = self.get_possible_knight_moves(king_piece, king_pos);
        if self.check_for_checks(knight_moves, king_color, &[PieceType::Knight], &mut result, &mut i) {
            return result;
        }
        let rook_moves = self.get_possible_rook_moves(king_piece, king_pos);
        if self.check_for_checks(rook_moves, king_color, &[PieceType::Rook, PieceType::Queen], &mut result, &mut i) {
            return result;
        }
        let bishop_moves = self.get_possible_bishop_moves(king_piece, king_pos);
        if self.check_for_checks(bishop_moves, king_color, &[PieceType::Bishop, PieceType::Queen], &mut result, &mut i) {
            return result;
        }
        let king_moves = self.get_possible_king_moves(king_piece, king_pos);
        if self.check_for_checks(king_moves, king_color, &[PieceType::King], &mut result, &mut i) {
            return result;
        }
        result
    }

    fn check_for_checks(&self, moves: MoveList, king_color: PieceColor, piece_type: &[PieceType], result: &mut [Option<usize>; 2], i: &mut usize) -> bool {
        for id in &moves.moves[0..moves.count] {
            if let Some(piece) = self.square[*id] {
                if piece.piece_color != king_color && piece_type.contains(&piece.piece_type) {
                    result[*i] = Some(*id);
                    *i += 1;
                    if *i == 2 { return true; } // Max 2 checkers, no need to continue
                }
            }
        }
        return false;
    }

    fn check_game_over(&mut self) -> bool {
        for idx in 0..64 {
            if let Some(piece) = self.square[idx] {
                if piece.piece_color == self.current_turn {
                    if let Ok(valid_moves) = self.get_valid_moves(idx) {
                        if valid_moves.count > 0 {
                            return false;                        }
                    }
                }
            }
        }
        true
    }

    /// Returns a tuple indicating if the game has concluded and the color of the winner.
    ///
    /// A return value of `(false, None)` indicates the game is active.
    /// `(true, Some(WinningColor))` indicates a checkmate.
    /// `(true, None)` indicates a stalemate.
    pub fn is_game_over(&self) -> (bool, Option<PieceColor>) {
        (self.game_over, self.winner)
    }

    pub fn get_current_turn(&self) -> PieceColor {
        self.current_turn
    }

    pub fn get_piece_color(&self, idx: usize) -> Option<PieceColor> {
        self.square[idx].map(|piece| piece.piece_color)
    }

    /// Calculates "pseudo-legal" moves for a piece based purely on its movement mechanics.
    ///
    /// Note: This method does NOT check if the resulting move leaves the King in check.
    /// It is primarily an internal helper for `get_valid_moves`.
    pub fn get_possible_moves(&self, idx: usize) -> Result<MoveList, &'static str> {
        let Some(piece) = self.square[idx] else {
            return Err("No piece at the given index");
        };

        match piece.piece_type {
            PieceType::Pawn => Ok(self.get_possible_pawn_moves(piece, idx, false)),
            PieceType::Knight => Ok(self.get_possible_knight_moves(piece, idx)),
            PieceType::Rook => Ok(self.get_possible_rook_moves(piece, idx)),
            PieceType::Bishop => Ok(self.get_possible_bishop_moves(piece, idx)),
            PieceType::Queen => Ok(self.get_possible_queen_moves(piece, idx)),
            PieceType::King => Ok(self.get_possible_king_moves(piece, idx)),
        }
    }

    /// Calculates strictly legal moves for a piece at the given index.
    ///
    /// This method generates all pseudo-legal moves and filters out any trajectory
    /// that would expose the player's own King to an active check by simulating "ghost moves".
    pub fn get_valid_moves(&mut self, idx: usize) -> Result<MoveList, &'static str> {
        let possible_moves = self.get_possible_moves(idx)?;
        let mut valid_moves = MoveList::new();
        let piece_color = self.square[idx].unwrap().piece_color; // Sure that a piece exists here

        // Making ghost moves and checking for checks in real time
        for i in 0..possible_moves.count {
            let target_idx = possible_moves.moves[i];
            let mut white_king_moved = false;
            let mut black_king_moved = false;

            let distance = target_idx as isize - idx as isize;

            if self.square[idx].unwrap().piece_type == PieceType::King && distance.abs() == 2 { // Castling Move
                let passing_idx = if distance > 0 { idx + 1 } else { idx - 1 };

                // Checking if the passing square is attacked (King can't passthrough an attacked square when castling)
                self.square[passing_idx] = self.square[idx].take();
                if idx == self.white_king_pos { 
                    self.white_king_pos = passing_idx;
                    white_king_moved = true;
                }
                if idx == self.black_king_pos {
                    self.black_king_pos = passing_idx;
                    black_king_moved = true;
                }

                let passing_safe = self.is_king_in_check(piece_color)[0].is_none();

                self.square[idx] = self.square[passing_idx].take(); // Move the king back to its original position
                if white_king_moved {
                    self.white_king_pos = idx;
                    white_king_moved = false;
                }
                if black_king_moved {
                    self.black_king_pos = idx;
                    black_king_moved = false;
                }

                if !passing_safe {
                    continue;
                }
            }

            let target_piece = self.square[target_idx];

            let mut ep_captured_idx = None;
            let mut ep_captured_piece = None;

            if self.square[idx].unwrap().piece_type == PieceType::Pawn {
                if let Some(ep_idx) = self.en_passant_target {
                    if ep_idx == target_idx {
                        ep_captured_idx = if piece_color == PieceColor::White { Some(target_idx - 8) } else { Some(target_idx + 8) };
                        ep_captured_piece = self.square[ep_captured_idx.unwrap()].take();
                    }
                }
            }

            self.square[target_idx] = self.square[idx].take();

            // Temporarily change king pos if its a king
            if idx == self.white_king_pos { 
                self.white_king_pos = target_idx;
                white_king_moved = true;
            }
            if idx == self.black_king_pos {
                self.black_king_pos = target_idx;
                black_king_moved = true;
            }

            let is_safe = self.is_king_in_check(piece_color)[0].is_none();

            if is_safe {
                valid_moves.push(target_idx);
            }

            self.square[idx] = self.square[target_idx].take();
            self.square[target_idx] = target_piece;

            if let Some(captured_idx) = ep_captured_idx {
                self.square[captured_idx] = ep_captured_piece; // Restore the captured pawn in case of en passant
            }

            // Restore King pos
            if white_king_moved { self.white_king_pos = idx; }
            if black_king_moved { self.black_king_pos = idx;  }
        }

        Ok(valid_moves)
    }

    fn get_possible_pawn_moves(&self, piece: Piece, idx: usize, checking_for_check: bool) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let r = row as isize;
        let c = col as isize;
        let row_step: isize = if piece.piece_color == PieceColor::White { 1 } else { -1 }; // Move up for white and down for black

        // Check Diagonally
        for col_step in [1, -1] {
            if (r + row_step) >= 0 && (r + row_step) <= 7 && (c + col_step) >= 0 && (c + col_step) <= 7 { // Bounds check
                let target_idx = (8 * (r+row_step) + (c + col_step)) as usize;
                if let Some(target_piece) = self.square[target_idx] {
                    if target_piece.piece_color != piece.piece_color { // Valid move if there's an opponent's piece
                        result.push(target_idx);
                    }
                } else if let Some(en_passant_idx) = self.en_passant_target {
                    if target_idx == en_passant_idx {
                        if let Some(target_piece) = self.square[if piece.piece_color == PieceColor::White { en_passant_idx - 8 } else { en_passant_idx + 8 }] {
                            if target_piece.piece_color != piece.piece_color && target_piece.piece_type == PieceType::Pawn {
                                result.push(target_idx);
                            }
                        }
                    }
                }
            }
        }

        if checking_for_check { return result; } // If we're just checking for checks, we don't need to check forward moves since pawns can't check kings with forward moves

        // Check forward
        for j in [1, 2] {
            let row_step = j*row_step;
            if (r + row_step) > 7 || (r + row_step) < 0 { // Out of bounds
                break;
            }
            let target_idx = 8*((r+row_step) as usize)+col;
            let None = self.square[target_idx] else { break; }; // Invalid move if there's a piece in the way
 
            // The pawn can move 2 steps forward only if it's in its starting position
            if j==2 && !((row == 1 && piece.piece_color == PieceColor::White) || (row == 6 && piece.piece_color == PieceColor::Black)) {
                break;
            }
            result.push(target_idx)
        }

        result
    }

    fn get_possible_knight_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)]; // All possible knight directional moves
        self.get_directional_moves(&mut result, piece, row, col, &directions, 1); // Checks only for 1 step in each direction since knights can only move 1 step in their unique L-shaped pattern
        result
    }

    fn get_possible_rook_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)]; // Straight directions
        self.get_directional_moves(&mut result, piece, row, col, &directions, 7); // Checks for up to 7 steps in each direction since the rook can move across the entire board
        result
    }

    fn get_possible_bishop_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        // Same as rook but diagonal directions
        let directions = [(1,1), (-1,1), (1,-1), (-1,-1)];
        self.get_directional_moves(&mut result, piece, row, col, &directions, 7);
        result
    }

    fn get_possible_queen_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        // Rook and Bishop Combined
        let directions = [(1,1), (-1,1), (1,-1), (-1,-1), (1, 0), (-1, 0), (0, 1), (0, -1)];
        self.get_directional_moves(&mut result, piece, row, col, &directions, 7);
        result
    }

    fn get_possible_king_moves(&self, piece: Piece, idx: usize) -> MoveList {
        let mut result = MoveList::new();
        let (row, col) = Board::index_to_coordinates(idx);
        let directions = [(1,1), (-1,1), (1,-1), (-1,-1), (1, 0), (-1, 0), (0, 1), (0, -1)]; // Directions same as queen
        self.get_directional_moves(&mut result, piece, row, col, &directions, 1); // But moves only 1 step

        // Check for castling
        if !self.checked { 
            if piece.piece_color == PieceColor::White {
                if self.castle_rights.white_kingside && self.square[5].is_none() && self.square[6].is_none() {
                    result.push(6);
                }
                if self.castle_rights.white_queenside && self.square[1].is_none() && self.square[2].is_none() && self.square[3].is_none() {
                    result.push(2);
                }
            } else {
                if self.castle_rights.black_kingside && self.square[61].is_none() && self.square[62].is_none() {
                    result.push(62);
                }
                if self.castle_rights.black_queenside && self.square[57].is_none() && self.square[58].is_none() && self.square[59].is_none() {
                    result.push(58);
                }
            }
        }
        result
    }

    fn get_directional_moves (&self, result: &mut MoveList, piece: Piece, row: usize, col: usize, directions: &[(isize, isize)], max_steps: usize) {
        let r = row as isize;
        let c = col as isize;

        // Iterate over every direction
        for &(row_step, col_step) in directions {
            let mut current_r = r + row_step;
            let mut current_c = c + col_step;

            let mut steps: usize = 0;
            // Keep moving until max steps is reached
            while current_r <= 7 && current_r >= 0 && current_c <= 7 && current_c >= 0 { // Bounds check
                let target_idx = (8 * current_r + current_c) as usize;

                match self.square[target_idx] {
                    Some(target_piece) => {
                        if target_piece.piece_color != piece.piece_color {
                            result.push(target_idx); // Valid move if opponent's piece
                        }
                        break; // Can't go forward
                    }
                    None => {
                        result.push(target_idx);
                        steps += 1;
                        if steps >= max_steps { break; } // Max steps reached
                        current_r += row_step;
                        current_c += col_step;
                    }
                }
            }
        }
    }

    pub fn get_piece_char(&self, idx: usize) -> Option<char> {
        match self.square[idx] {
            Some(piece) => {
                let char = match piece.piece_type {
                            PieceType::Pawn => 'p',
                            PieceType::Knight => 'n',
                            PieceType::Bishop => 'b',
                            PieceType::Rook => 'r',
                            PieceType::Queen => 'q',
                            PieceType::King => 'k',
                };
                // Small for black, Capital for white
                Some(match piece.piece_color {
                    PieceColor::White => char.to_ascii_uppercase(),
                    _ => char
                })
            },
            None => None,
        }
    }
}

// For debugging only
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         for i in 0..8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_enforcement() {
        let mut board = Board::new();
        assert!(board.move_piece(12, 28).is_ok());
        assert!(board.move_piece(13, 29).is_err());
        assert!(board.move_piece(52, 36).is_ok());
    }

    #[test]
    fn en_passant() {
        let mut board = Board::new();
        board.move_piece(12, 28).unwrap();
        board.move_piece(48, 40).unwrap();
        board.move_piece(28, 36).unwrap();
        board.move_piece(51, 35).unwrap();
 
        assert!(board.move_piece(36, 43).is_ok());
        assert!(board.square[35].is_none());
        assert_eq!(board.square[43].unwrap().piece_type, PieceType::Pawn);
    }

    #[test]
    fn castling_denial() {
        let mut board = Board::new();
        board.move_piece(12, 28).unwrap();
        board.move_piece(52, 36).unwrap();
        board.move_piece(13, 29).unwrap();
        board.move_piece(36, 29).unwrap();
        board.move_piece(5, 26).unwrap();
        board.move_piece(51, 43).unwrap();
        board.move_piece(6, 21).unwrap();
        board.move_piece(59, 31).unwrap();
 
        assert!(board.move_piece(4, 6).is_err());
    }

    #[test]
    fn checkmate_detection() {
        let mut board = Board::new();
        board.move_piece(13, 21).unwrap();
        board.move_piece(52, 36).unwrap();
        board.move_piece(14, 30).unwrap();
        board.move_piece(59, 31).unwrap();

        let (game_over, winner) = board.is_game_over();
        assert!(game_over);
        assert_eq!(winner, Some(PieceColor::Black));
    }

    #[test]
    fn auto_queen_promotion() {
        let mut board = Board::new();
        board.square[63] = None;
        board.square[55] = Some(Piece::new(PieceType::Pawn, PieceColor::White));
        board.square[15] = None;
 
        board.move_piece(55, 63).unwrap();
 
        assert_eq!(board.square[63].unwrap().piece_type, PieceType::Queen);
    }
}
