use tty_mate::Board;

fn main() {
    let mut board = Board::new();
    println!("{}", board);
    if let Err(e)= board.move_piece(8, 24) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(49, 33) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(24, 33) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(0, 48) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(9, 17) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(2, 16) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(16, 52) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(3, 2) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(2, 16) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(16, 34) {
        println!("{e}");
    }
    println!("{}", board);
    if let Err(e)= board.move_piece(34, 50) {
        println!("{e}");
    }
    println!("{}", board);
}
