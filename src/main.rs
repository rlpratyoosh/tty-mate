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
}
