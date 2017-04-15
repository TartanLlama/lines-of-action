use std::sync::mpsc::{Sender};
use super::command::{CommandResponse,Move};
use super::board::Colour;

#[derive(Clone)]
pub enum PlayerState {
    Default,
    WaitingOnMove,
}

pub struct Player {
    pub name: String,
    pub ready: bool,
    pub reply: Option<Sender<CommandResponse>>,
    pub colour: Colour,
    pub move_cache: Option<Move>,
    pub state: PlayerState,
}

impl Player {
    pub fn new (name: String) -> Player {
        Player {
            name: name,
            ready: false,
            reply: None,
            colour: Colour::White,
            move_cache: None,
            state: PlayerState::Default,
        }
    }
}
