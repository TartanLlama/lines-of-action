extern crate sfml;
extern crate lines_of_action;

#[macro_use] extern crate log;
extern crate env_logger;

use std::collections::btree_map::BTreeMap;

use lines_of_action::board::{Board,SCREEN_SIZE,Colour,MoveErr};
use lines_of_action::draw::Drawable;
use lines_of_action::server::GameServer;
use lines_of_action::command::{GetCommandErr,CommandProducer,CommandData,CommandErr,CommandResponse,CommandOk,Move};
use lines_of_action::player::{Player,PlayerState};

use std::sync::mpsc::Sender;

use sfml::window::{ContextSettings, VideoMode, event, Close};
use sfml::window::keyboard::Key;
use sfml::graphics::{RenderWindow, RenderTarget, Color};

struct Engine {
    players: BTreeMap<i32,Player>,
    board: Board,
    server: GameServer,
    window: RenderWindow,
    turn: Colour,
    game_started: bool,
    send_moves: bool,
}

impl Engine {
    fn get_other_player_mut (&mut self, id: i32) -> Option<&mut Player> {
        for i in self.players.keys().cloned().collect::<Vec<_>>() {
            if id != i {
                return Some(self.players.get_mut(&i).unwrap());
            }
        }
        return None;
    }
    
    fn new() -> Engine {
        let mut window = match RenderWindow::new(VideoMode::new_init(SCREEN_SIZE, SCREEN_SIZE, 32),
                                                 "Lines of Action",
                                                 Close,
                                                 &ContextSettings::default()) {
            Some(window) => window,
            None => panic!("Cannot create a new Render Window.")
        };

        let mut board = Board::new();
        let mut players = BTreeMap::new();
        let server = GameServer::new();

        Engine {
            players: players,
            board: board,
            server: server,
            window: window,
            turn: Colour::White,
            game_started: false,
            send_moves: false,
        }
    }

    fn run(&mut self){
        // Create the window of the application
        while self.window.is_open() {
            // Handle events
            for event in self.window.events() {
                match event {
                    event::Closed => self.window.close(),
                    event::KeyPressed{code: c,..} => self.handle_key_press(c),
                    _             => {/* do nothing */}
                }
            }

            match self.server.get_command() {
                Ok(mut command) =>
                {
                    match command.data {
                        CommandData::Move(id,mov) =>
                        self.move_piece(id,&mut command.reply,&mov),
                        CommandData::GetMove(id) =>
                        self.get_move(id, &mut command.reply),
                        CommandData::Register(reg,id) =>
                        self.register_player(&mut command.reply,&reg,id),
                        CommandData::Message(id,msg) =>
                        self.display_message(&mut command.reply,&msg),
                        CommandData::Ready(id) =>
                        self.handle_ready(&mut command.reply,id),
                            //_ => Err("Unhandled command".to_string()),
                    }
                },

                Err(GetCommandErr::Failed(why)) => panic!(why),
                Err(GetCommandErr::NoCommands) => (),
            }

            // Clear the window
            let background = Color::new_rgb(75,45,25);
            self.window.clear(&background);

            match self.board.draw(&mut self.window) {
                Err(s) => panic!(s),
                Ok(()) => {},
            };

            // Display things on screen
            self.window.display();
        }
    }

    fn get_move (&mut self, id: i32, reply: &mut Sender<CommandResponse>)
    {
        let ref mut player = self.players.get_mut(&id).unwrap();
        debug!("{} requested a move.", player.name.clone());

        if let Some(mov) = player.move_cache.clone() {
            debug!("{} already has a move waiting.", player.name.clone());
            if self.send_moves {
                debug!("Sending move.");
                reply.send(Ok(CommandOk::Move(mov))).unwrap();
                self.send_moves = false;
                player.move_cache = None;
            } else {
                debug!("{} waiting on move.", player.name.clone());
                player.reply = Some(reply.clone());
                player.state = PlayerState::WaitingOnMove;
            }
        } else {
            debug!("{} waiting on move.", player.name.clone());
            player.reply = Some(reply.clone());
            player.state = PlayerState::WaitingOnMove;
        }
    }
    
    fn move_piece (&mut self, id: i32, reply: &mut Sender<CommandResponse>, mov: &Move) {
        debug!("{} sent a move.", self.players.get(&id).unwrap().name);
        let player_colour = self.players.get(&id).unwrap().colour;

        reply.send(match self.board.move_piece(&mov,&player_colour) {
            Ok(()) => {
                Ok(CommandOk::None)
            },
            Err(MoveErr::NoPiece) => Err(CommandErr::NoPiece),
            Err(MoveErr::WrongPiece) => Err(CommandErr::WrongPiece),
        });

        let mut send_moves = self.send_moves.clone();
        {
            let ref mut other = self.get_other_player_mut(id).unwrap();
            if let PlayerState::WaitingOnMove = other.state {
                if send_moves {
                    debug!("{} is waiting on move.", other.name.clone());
                    debug!("Sending move.");
                    other.reply.clone().unwrap().send(Ok(CommandOk::Move(mov.clone())));
                    other.state = PlayerState::Default;
                    send_moves = false;
                } else {
                    debug!("Caching move");
                    other.move_cache = Some(mov.clone());
                }
            } else {
                debug!("Caching move");
                other.move_cache = Some(mov.clone());
            }
        }
        self.send_moves = send_moves;
    }
    
    fn register_player(&mut self, reply: &mut Sender<CommandResponse>, name: &str, id: i32) {
        if self.players.contains_key(&id) {
            reply.send(Err(CommandErr::AlreadyRegistered));
            return;
        }

        reply.send(
            if self.players.len() < 2 {
                self.players.insert(id,(Player::new(name.to_string())));
                debug!("Registered new player: {}", name);
                Ok(CommandOk::None)
            } else {
                Err(CommandErr::GameFull)
            });
    }

    fn display_message(&self, reply: &mut Sender<CommandResponse>, msg: &str) {
        println!("{}",msg);
        reply.send(Ok(CommandOk::None));
    }

    fn handle_ready(&mut self, reply: &mut Sender<CommandResponse>, id: i32) {
        match self.players.get_mut(&id) {
            Some(player) => player.ready = true,
            None => { reply.send(Err(CommandErr::NotRegistered)); return; },
        }

        self.players.get_mut(&id).unwrap().reply = Some(reply.clone());

        if self.players.len() < 2 {
            return;
        }
        
        for (id, p) in &self.players {
            if !p.ready {
                return;
            }
        }

        debug!("Ready to play!");
        
        let keys = self.players.keys().cloned().collect::<Vec<_>>();
        
        {
            let p2_name = self.players.get(&keys[1]).unwrap().name.clone();
            let ref mut p1 = &mut self.players.get_mut(&keys[0]).unwrap();
            p1.colour = Colour::White;
            p1.reply.clone().unwrap().send(Ok(CommandOk::Ready(p2_name, p1.colour.clone())));
        }

        {
            let p1_name = self.players.get(&keys[0]).unwrap().name.clone();
            let ref mut p2 = &mut self.players.get_mut(&keys[1]).unwrap();
            p2.colour = Colour::Black;
            p2.reply.clone().unwrap().send(Ok(CommandOk::Ready(p1_name, p2.colour)));
        }
    }

    fn handle_key_press(&mut self, code: Key) {
        debug!("Key pressed");
        if code == Key::Space {
            debug!("It was space, checking available moves");
            for i in self.players.keys().cloned().collect::<Vec<_>>() {
                debug!("Checking player {}", i);
                if let PlayerState::WaitingOnMove = self.players.get(&i).unwrap().state.clone() {
                    debug!("They are waiting on a move");
                    if let Some(mov) = self.players.get(&i).unwrap().move_cache.clone() {
                        debug!("Sending move.");
                        self.players.get(&i).unwrap().reply.clone().unwrap().send(Ok(CommandOk::Move(mov.clone())));
                        self.players.get_mut(&i).unwrap().state = PlayerState::Default;
                        self.players.get_mut(&i).unwrap().move_cache = None;
                        self.send_moves = false;
                        
                        return;
                    }
                }
            }
            self.send_moves = true;
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    
    let mut engine = Engine::new();
    engine.run();
}
