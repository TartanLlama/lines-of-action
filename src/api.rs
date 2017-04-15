extern crate regex;

use self::regex::{Regex,Captures};
use std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use std::io::{Read,Write};
use super::server::PORT;

pub use super::command::Move;
pub use super::board::Colour;

///API for communicating with the lines of action server.
///
///The server can be communicated with directly using telnet. The protocol is documented [here](../server/index.html#protocol)
pub struct API {
    connection: TcpStream,
}

impl API {
    fn read_message (stream: &mut TcpStream) -> String {
        let mut c = [0;1];
        let mut buf = Vec::new();

        stream.read(&mut c);
        while c[0] != ';' as u8 {
            buf.push(c[0]);
            stream.read(&mut c);
        }
        buf.push(c[0]);
        stream.read(&mut c);

        String::from_utf8(buf).unwrap()
    }
    
    ///Connect to the server at the given socket address
    pub fn new<A: ToSocketAddrs> (addr: A) -> API {
        let connection = TcpStream::connect(addr).unwrap();

        API {
            connection: connection,
        }
    }

    ///Register as a player with the server
    pub fn register (&mut self, name: &str) {
        self.connection.write(
            format!("Hello, my name is {}, you killed my father, prepare to die;",
                    name)
            .as_bytes()).unwrap();
        API::read_message(&mut self.connection);
    }

    ///Move a piece from (mov.sx,mov.sy) to (mov.dx,mov.dy)
    pub fn move_piece (&mut self, mov: &Move) {
        self.connection.write(
            format!("({},{}) -> ({},{});",
                    mov.sx,mov.sy,mov.dx,mov.dy)
            .as_bytes()).unwrap();
        API::read_message(&mut self.connection);
    }

    //Get the opponent's next move
    pub fn get_move (&mut self) -> Move {
        self.connection.write(b"Gimmeh!;").unwrap();
        let msg = API::read_message(&mut self.connection);
        
        let move_re = Regex::new(r"^\((\d),(\d)\) ?-> ?\((\d),(\d)\);")
            .ok()
            .expect("Failed to create regex");
        
        let caps = move_re.captures(&msg)
            .expect("Failed to parse regex")
            .iter()
            .skip(1)
            .map(|x:Option<&str>| { x.unwrap().parse::<u8>().unwrap() })
            .collect::<Vec<_>>();

        Move::new_vec(caps)
    }

    ///Waits until an opponent is ready. Returns your colour and the opponent's name.
    pub fn wait_on_start (&mut self) -> (Colour, String) {
        self.connection.write(b"Bring it, yo;").unwrap();
        let msg = API::read_message(&mut self.connection);

        let reply_re = Regex::new(r"You are ([A-Za-z]+) and ([A-Za-z0-9]+) wants to batter you;")
            .ok()
            .expect("Bad regex");

        let caps = reply_re.captures(&msg)
            .expect("Regex didn't match");

        let colour = caps.at(1).unwrap().parse::<Colour>()
            .ok()
            .expect("Couldn't parse Colour");

        (colour, caps.at(2).unwrap().to_string())
    }

    ///Get the default port the server listens on
    pub fn default_server_port() -> u16 {
        return PORT;
    }
}
