//! Built-in game implementations for NT-GAME.
//!
//! Games are categorized by difficulty and constellation level.

pub mod hex_tictactoe;
pub mod game_2048;

pub use hex_tictactoe::HexTicTacToe;
pub use game_2048::Game2048;
