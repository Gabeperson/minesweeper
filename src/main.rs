use tokio;
use rocket;
use include_dir::include_dir;
use include_dir::Dir;
use serde_json;
use serde::{Serialize, Deserialize};
use dolphine;
use dolphine::Dolphine;
use rand::thread_rng;
use rand::seq::SliceRandom;

static FILES: Dir = dolphine::include_dir!("web");


#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Mine,
    None,
}

#[derive(Clone, Debug)]
struct MineSweeper {
    numbers_board: Vec<Vec<u8>>,
    revealed_board: Vec<Vec<bool>>,
}

impl MineSweeper {
    fn new(x: isize, y: isize, mines: usize) -> MineSweeper {
        let mut board: Vec<Vec<Tile>> = vec![vec![Tile::None; x as usize]; y as usize];
        let total_length = x * y;
        let array_length = total_length - mines as isize;
        let mut array = vec![Tile::None; (array_length) as usize];
        array.extend_from_slice(&vec![Tile::Mine; mines]);
        array.shuffle(&mut thread_rng());
        for y_value in 0..y {
            for x_value in 0..x {
                board[y_value as usize][(x_value) as usize] = array[(y_value*x+x_value) as usize];
            }
        }
        let mut numbers_board: Vec<Vec<u8>> = vec![vec![0; x as usize]; y as usize];
        for y_value in 0..y {
            for x_value in 0..x {
                if board[y_value as usize][x_value as usize] == Tile::None {
                    continue;
                }
                numbers_board[y_value as usize][x_value as usize] = 100;
                for y1 in -1..=1 {
                    for x1 in -1..=1 {
                        if y_value+y1 >= y || y_value+y1 < 0 || x_value + x1 >= x || x_value+x1 < 0 {
                            continue;
                        }
                        numbers_board[(y_value+y1) as usize][(x_value+x1) as usize] += 1;
                    }
                }
            }
        }
        MineSweeper {
            numbers_board,
            revealed_board: vec![vec![false; x as usize]; y as usize]
        }


        
    }
    fn debugprint(&self) {
        for y in self.numbers_board.iter() {
            let mut s = String::new();
            for x in y.iter() {
                s.push_str(&(x.to_string() + ","));
            }
            println!("{}", s);
        }
    }

}

fn main() {
    let m = MineSweeper::new(20, 20, 50);
    m.debugprint()
}