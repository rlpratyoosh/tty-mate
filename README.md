```text
 _____ _____ ___  _      _     ____ _____ _____
/__ __Y__ __\\  \//     / \__/|/  _ Y__ __Y  __/
  / \   / \   \  /_____ | |\/||| / \| / \ |  \  
  | |   | |   / / \____\| |  ||| |-|| | | |  /_ 
  \_/   \_/  /_/        \_/  \|\_/ \| \_/ \____\

```

A minimal, terminal-based chess game written in Rust.

**tty-mate** lets you play local two-player chess directly in your terminal. It uses a clean interface built with `ratatui` and focuses on just playing the game without distractions.

## Features

* **Full FIDE Rule Enforcement:** The engine correctly handles standard piece movement, Check, Checkmate, Stalemate, Castling, and En Passant.
* **Terminal UI:** Split view showing the interactive board and an auto-scrolling move history panel.
* **Algebraic Notation:** Automatically records moves in standard chess notation.

## Current Limitations

While the core game is fully playable, a couple of specific edge cases are still being worked on:

* **Pawn Promotion:** Currently, pawns automatically promote to a Queen when reaching the end of the board. A menu to choose the promotion piece is planned.
* **Draw by Repetition:** The game does not yet automatically detect a draw if the exact same board state repeats three times.

## How to Install and Run

Make sure you have [Rust and Cargo](https://rustup.rs/) installed on your system.

1. Clone the repository:
```bash
git clone https://github.com/rlpratyoosh/tty-mate.git
cd tty-mate

```


2. Run the game:
```bash
cargo run --release

```



*Note: Use `j/k/h/l` or arrow keys to navigate the menu and board. Press `Enter` to select/move a piece, and `Esc` to cancel a selection.*

## Future Roadmap

* **Multiplayer Architecture:** Building out a broker server system to allow playing against friends over the network instead of just local hot-seat play.

## Contributing

This is a personal portfolio project built for my own learning and growth. As such, **I am not accepting pull requests or external contributions at this time.** However, you are more than welcome to fork the repository, read the code, and experiment with it on your own!
