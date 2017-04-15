extern crate sfml;
use self::sfml::graphics::{RenderWindow,RectangleShape,RenderTarget,Color,CircleShape};
use self::sfml::system::vector2::Vector2f;

use super::board::{Board,CELL_SIZE,Colour};

pub trait Drawable {
    fn draw (&self, window: &mut RenderWindow) -> Result<(),String>;
}


impl Drawable for Board {
    fn draw (&self, window: &mut RenderWindow) -> Result<(),String>{
        let mut rect = RectangleShape::new().unwrap();
        rect.set_fill_color(&Color::transparent());
        rect.set_outline_color(&Color::black());
        rect.set_outline_thickness(1.0);
        rect.set_size2f(CELL_SIZE as f32,CELL_SIZE as f32);

        let mut piece = CircleShape::new().unwrap();
        piece.set_radius((CELL_SIZE/2) as f32);
        
        for (ri,row) in self.cells.iter().enumerate() {
            for (ci,cell) in row.iter().enumerate() {
                let position = Vector2f::new((ri*(CELL_SIZE as usize)) as f32, (ci*(CELL_SIZE as usize)) as f32);
                rect.set_position(&position);
                window.draw(&rect);

                if cell.has_piece {
                    match cell.colour {
                        Colour::White => piece.set_fill_color(&Color::white()),
                        Colour::Black => piece.set_fill_color(&Color::black()),
                    };
                    piece.set_position(&position);
                    window.draw(&piece);
                }
            }
        }
        
        Ok(())
    }
}
