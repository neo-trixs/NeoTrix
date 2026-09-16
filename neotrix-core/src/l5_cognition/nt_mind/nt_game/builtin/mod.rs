//! Built-in game implementations for NT-GAME.
//!
//! Games are categorized by difficulty and constellation level.

pub mod game_2048;
pub mod hex_tictactoe;

pub use game_2048::Game2048;
pub use hex_tictactoe::HexTicTacToe;
