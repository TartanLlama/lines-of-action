extern crate lines_of_action;

use lines_of_action::api::{API,Move,Colour};

fn main() {
    let mut api = API::new(("127.0.0.1", API::default_server_port()));
    api.register("Robot");
    let (colour, opp) = api.wait_on_start();
    println!("{} is getting smashed.",opp);

    if colour == Colour::White {
        loop {
            println!("Waiting");
            api.get_move();
            println!("Got");
            let mov = Move::new(0,1,1,1);
            api.move_piece(&mov);
            println!("Waiting");
            api.get_move();
            println!("Got");
            let mov = Move::new(1,1,0,1);
            api.move_piece(&mov);
        }
    } else {
        loop {
            let mov = Move::new(1,7,1,6);
            api.move_piece(&mov);
            println!("Waiting");
            api.get_move();
            println!("Got");       
            let mov = Move::new(1,6,1,7);
            api.move_piece(&mov);
            println!("Waiting");
            api.get_move();
            println!("Got");
        }
    }
}
