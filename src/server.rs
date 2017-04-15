//! The networking part of the lines of action server.
//! # Protocol
//! Optional parts are marked `[]`. Whitespace is important. Non-literals are marked `%`.
//! `You wot m8?;` is returned for an unrecognised command.
//!
//! ## Register player:
//! `Hello, my name is %name[, you killed my father, prepare to die];`
//!
//! ### Returns
//! `Oh, hai %name!;` - success
//!
//! `You've already registered, you asshat;` - this TCP connection has already registered
//!
//! `The game is full, get tae;` - the game already has two players registered
//!
//! ## Move piece:
//! `(%sx,%sy)[ ]->[ ](%dx,%dy);`
//!
//! ### Returns:
//! Nothing, yet
//!
//! ## Send message to be displayed by the server:
//! `"%message";`
//!
//! ### Returns:
//! Nothing
//!
//! ## Wait for game start:
//! `Bring it, yo;`
//!
//! ### Returns:
//! `You are %colour and %opponent wants to batter you;` - game has started
//! `Who even are you?;` - you have not yet registered
//!
//! ## Get opponent's move:
//! `Gimmeh;`
//!
//! ### Returns:
//! Nothing, yet

extern crate regex;

use super::command::{Move,Command,CommandProducer,GetCommandErr,CommandResponse,CommandData,CommandErr,CommandOk};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::thread;
use std::str;
use self::regex::{Regex,Captures};
use std::io::{Read,Write};
use std::sync::mpsc::{channel,Sender,Receiver,TryRecvError};

pub const PORT: u16 = 1234;

fn handle_message(buf: &[u8], command_sender: &Sender<Command>,
                  response_send: &Sender<CommandResponse>,
                  response_recv: &Receiver<CommandResponse>,
                  stream: &mut TcpStream, id: i32) {
    let move_re = Regex::new(r"^\((\d),(\d)\) ?-> ?\((\d),(\d)\);").unwrap();
    let register_re = Regex::new(r"^Hello, my name is ([A-Za-z0-9]+)(, you killed my father, prepare to die)?;").unwrap();
    let message_re = Regex::new("^\"(.+)\";").unwrap();
    let get_re = Regex::new("^Gimmeh!;").unwrap();
    let ready_re = Regex::new("^Bring it, yo;").unwrap();

    let buf = str::from_utf8(buf).unwrap();

    //This is a move
    if move_re.is_match(&buf) {
        //Transform captures into vector of u8s
        let caps = move_re.captures(&buf).unwrap()
            .iter()
            .skip(1)
            .map(|x:Option<&str>| { x.unwrap().parse::<u8>().unwrap() })
            .collect::<Vec<_>>();

        let mov = CommandData::Move(id,Move::new_vec(caps));
        let command = Command{ data: mov,
                              reply: response_send.clone()};

        command_sender.send(command).unwrap();
        
        match response_recv.recv().unwrap() {
            Ok(CommandOk::None) =>
            { stream.write(b"Move successful;\n"); },
            Err(CommandErr::NoPiece) =>
            { stream.write(b"You can't move air, ya numpty;\n"); },
            Err(CommandErr::WrongPiece) =>
            { stream.write(b"Stick to your own pieces!;\n"); },
            _ => {},
        }
    }

    //This is a registration
    else if register_re.is_match(&buf) {
        let name = register_re.captures(&buf).unwrap().at(1).unwrap();
        let reg = CommandData::Register(name.to_string(),id);
        let command = Command{ data: reg,
                              reply: response_send.clone()};

        command_sender.send(command).unwrap();
        match response_recv.recv().unwrap() {
            Ok(CommandOk::None) =>
            { stream.write(format!("Oh, hai {}!;\n",name).as_bytes()); },
            Err(CommandErr::AlreadyRegistered) =>
            { stream.write(b"You've already registered, you asshat;\n"); },
            Err(CommandErr::GameFull) =>
            { stream.write(b"The game is full, get tae;\n"); },
            Err(CommandErr::Other(s)) => panic!(s),
            _ => panic!(),
        };
    }

    //This is a message
    else if message_re.is_match(&buf) {
        let msg = CommandData::Message(
            id,
            message_re.captures(&buf).unwrap().at(1).unwrap().to_string()
            );
        let command = Command{ data: msg,
                              reply: response_send.clone()};

        command_sender.send(command).unwrap();
        response_recv.recv().unwrap().unwrap();
    }

    //This is a request for a move
    else if get_re.is_match(&buf) {
        let command = Command{ data: CommandData::GetMove(id),
                              reply: response_send.clone()};
        command_sender.send(command).unwrap();
        
        match response_recv.recv().unwrap() {
            Ok(CommandOk::Move(mov)) =>
            { stream.write(format!("({},{})->({},{});\n", mov.sx,mov.sy,mov.dx,mov.dy).as_bytes()).unwrap(); },
            _ => panic!("Error in getting move"),
        }
    }

    //User is ready to start
    else if ready_re.is_match(&buf) {
        let command = Command{ data: CommandData::Ready(id),
                              reply: response_send.clone()};
        command_sender.send(command).unwrap();
        
        match response_recv.recv().unwrap() {
            Ok(CommandOk::None) => {},
            Ok(CommandOk::Ready(opponent,col)) =>
            { stream.write(format!("You are {:?} and {} wants to batter you;\n",col,opponent).as_bytes()); },
            Err(CommandErr::NotRegistered) =>
            { stream.write(b"Who even are you?;\n"); },
            _ => panic!(),
        }
    }


    //User is drunk
    else {
        stream.write(b"You wot m8?;\n");
    }
}

fn handle_connection(mut stream: TcpStream, command_sender: Sender<Command>, id: i32) {
    debug!("Accepted");
    let mut buf = [0 as u8;256];

    //Used to get errors back from the engine
    let (response_send,response_recv) = channel();

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(_) => handle_message(&buf, &command_sender,
                                    &response_send, &response_recv,
                                    &mut stream, id),
            Err(e) => panic!(e),
        }
    }
}

pub struct GameServer {
    command_receiver: Receiver<Command>,
    handle: thread::JoinHandle<()>,
}

impl GameServer {
    pub fn new() -> GameServer {
        let (send,recv) = channel();

        let handle = thread::spawn(move || {
            let mut id = 0;
            let listener = TcpListener::bind(("127.0.0.1",PORT))
                .ok()
                .expect(&format!("Could not bind to port {}", PORT));
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let send = send.clone();
                        id += 1;
                        thread::spawn(move|| {
                            handle_connection(stream, send, id);
                        });
                    }
                    Err(e) => { panic!(e); }
                }
            }
        });

        GameServer {
            command_receiver: recv,
            handle: handle,
        }
    }
}

impl CommandProducer for GameServer {
    fn get_command(&self) -> Result<Command,GetCommandErr> {
        match self.command_receiver.try_recv() {
            Ok(command) => Ok(command),
            Err(TryRecvError::Disconnected) => Err(GetCommandErr::Failed("Disconnected".to_string())),
            Err(TryRecvError::Empty) => Err(GetCommandErr::NoCommands),
        }
    }
}
