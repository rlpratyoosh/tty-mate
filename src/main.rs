use tty_mate::Board;

fn main() {
    let board = Board::new();
    println!("{}", board);
    println!("{:?}", board.get_possible_moves(49));
}
