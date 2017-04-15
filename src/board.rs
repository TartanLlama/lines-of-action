use super::command::{Move};
use std::fmt::{Formatter,Error,Display};
use std::str::FromStr;

pub const CELL_SIZE:u8 = 30;
pub const BOARD_SIZE:u8 = 8;
pub const SCREEN_SIZE:u32 = (BOARD_SIZE*CELL_SIZE) as u32;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Colour {
    White, Black
}

impl Colour {
    pub fn other(&self) -> Colour {
        match *self {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

impl FromStr for Colour {
    type Err = ();

    fn from_str(s: &str) -> Result<Colour, ()> {
        match s {
            "White" => Ok(Colour::White),
            "Black" => Ok(Colour::Black),
            _ => Err(()),
        }
    }
}

#[derive(Clone,Copy)]
pub struct Cell {
    pub has_piece: bool,
    pub colour: Colour,
}

impl Cell {
    fn white() -> Cell { Cell {has_piece: true, colour: Colour::White} }
    fn black() -> Cell { Cell {has_piece: true, colour: Colour::Black} }
}

impl Default for Cell {
    fn default() -> Cell {Cell {has_piece: false, colour: Colour::White} }
}

pub struct Board {
    pub cells: [[Cell; 8]; 8],
}

pub enum MoveErr {
    NoPiece,
    WrongPiece,
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            cells: [[Cell::default(); 8]; 8]
        };

        for i in 1..(BOARD_SIZE-1) {
            board.cells[(BOARD_SIZE-1) as usize][i as usize] = Cell::white();
            board.cells[0][i as usize] = Cell::white();

            board.cells[i as usize][(BOARD_SIZE-1) as usize] = Cell::black();
            board.cells[i as usize][0] = Cell::black();
        }
        board
    }

    pub fn move_piece (&mut self, mov: &Move, player_colour: &Colour) -> Result<(),MoveErr> {
        if !self.cells[mov.sx as usize] [mov.sy as usize].has_piece {
            Err(MoveErr::NoPiece)
        } else if self.cells[mov.sx as usize] [mov.sy as usize].colour != *player_colour {
            Err(MoveErr::WrongPiece)
        } else {
            self.cells[mov.dx as usize][mov.dy as usize] = self.cells[mov.sx as usize][mov.sy as usize];
            self.cells[mov.sx as usize][mov.sy as usize] = Cell::default();
            Ok(())
        }
    }

    pub fn colour_at (&self, x:usize, y:usize) -> Colour {
        self.cells[x][y].colour
    }
}
