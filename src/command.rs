use std::sync::mpsc::Sender;
use super::board::Colour;

pub enum GetCommandErr {
    NoCommands,
    Failed(String),
}

pub trait CommandProducer {
    fn get_command(&self) -> Result<Command, GetCommandErr>;
}

#[derive(Clone,Debug)]
pub struct Move {
    pub sx:u8,
    pub sy:u8,
    pub dx:u8,
    pub dy:u8,
}

impl Move {
    pub fn new(sx:u8,sy:u8,dx:u8,dy:u8) -> Move {
        Move { sx:sx, sy:sy, dx:dx, dy:dy }
    }

    pub fn new_vec(v: Vec<u8>) -> Move {
        assert_eq!(v.len(), 4);
        Move { sx:v[0], sy:v[1], dx:v[2], dy:v[3] }
    }
}

pub enum CommandData {
    Move (i32,Move),
    GetMove(i32),
    Ready(i32),
    Register(String,i32),
    Message(i32,String),
}

#[derive(Debug)]
pub enum CommandErr {
    AlreadyRegistered,
    NotRegistered,
    GameFull,

    NoPiece,
    WrongPiece,
    
    Other(String),
}

#[derive(Debug)]
pub enum CommandOk {
    Ready(String, Colour),
    Move(Move),
    None,
}


pub type CommandResponse = Result<CommandOk,CommandErr>;

pub struct Command {
    pub data: CommandData,
    pub reply: Sender<CommandResponse>,
}
