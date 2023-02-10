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
use std::io;
use std::time::Instant;

static FILES: Dir = dolphine::include_dir!("web");
// todo bitwise shift for indications???


#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Mine,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TileMarking {
    Mine,
    Question,
    None,
}

#[derive(Clone, Debug)]
struct MineSweeper {
    numbers_board: Vec<Vec<u8>>,
    revealed_board: Vec<Vec<bool>>,
    flagged_board: Vec<Vec<TileMarking>>
}

#[derive(Clone, Debug)]
enum GameResult {
    Continuing,
    Won,
    Lost,
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
            revealed_board: vec![vec![false; x as usize]; y as usize],
            flagged_board: vec![vec![TileMarking::None; x as usize]; y as usize]
        }


        
    }
    fn debugprint(&self) {
        for y in self.numbers_board.iter() {
            let mut s = String::from("[");
            for x in y.iter() {
                s.push_str(&(x.to_string() + ", "));
            }
            s.push_str("]");
            println!("{}", s);
        }
        println!("--------------");
        for y in self.revealed_board.iter() {
            let mut s = String::from("[");
            for x in y.iter() {
                if *x {
                    s.push_str(&(1.to_string() + ", "));
                    continue
                }
                s.push_str(&(0.to_string() + ", "));
                
            }
            s.push_str("]");
            println!("{}", s);
        }
    }

    fn reveal(&mut self, y: usize, x: usize) {
        let mut stack = Vec::new();
        if self.numbers_board[y][x] > 0 {
            self.revealed_board[y][x] = true;
            return;
        }
        if self.revealed_board[y][x] {
            return;
        }
        stack.push((y as i16, x as i16, 0));
        // REMOVE C
        self.reveal_impl(&mut stack, (self.numbers_board.len() as i16, self.numbers_board[0].len() as i16));
        // REMOVE
    }
    
    fn reveal_impl(&mut self, stack: &mut Vec<(i16, i16, u8)>, sizes: (i16, i16)) {
        const CELL_8_ARRAY: &'static [(i16, i16); 9] = &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (0, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        while let Some((y_coord, x_coord, arr_index)) = stack.pop() {
            // check to see if we're at end. If so, remove from stack
            if arr_index == 9 {
                continue;
            }
            // run check for bug in debug mode
            #[cfg(debug_assertions)]
            if arr_index > 9 {
                panic!("Higher than 9?? WHAT???");
            }
            
            let (y_offset, x_offset) = CELL_8_ARRAY[arr_index as usize];
            if y_coord + y_offset >= sizes.0 || y_coord + y_offset < 0 || x_coord + x_offset >= sizes.0 || x_coord + x_offset < 0 {
                stack.push((y_coord, x_coord, arr_index+1));
                continue;
            }
            if self.revealed_board[(y_coord + y_offset) as usize][(x_coord+x_offset) as usize] {
                stack.push((y_coord, x_coord, arr_index+1));
                continue;
            }
            if self.numbers_board[(y_coord + y_offset) as usize][(x_coord+x_offset) as usize] > 0 {
                // prevent unnecessary stack calling, etc w/ non-0 tiles
                self.revealed_board[(y_coord + y_offset) as usize][(x_coord+x_offset) as usize] = true;
                stack.push((y_coord, x_coord, arr_index+1));
                continue;
            }
            self.revealed_board[(y_coord + y_offset) as usize][(x_coord+x_offset) as usize] = true;
            stack.push((y_coord, x_coord, arr_index+1));
            stack.push((y_coord+y_offset, x_coord+x_offset, 0));            
        }

    }

    fn chord(&mut self, y: i16, x: i16) {
        let mut c = 0;
        let n = self.numbers_board[y as usize][x as usize];
        for y_offset in -1..=1 {
            for x_offset in -1..=1 {
                if y_offset == 0 && x_offset == 0 {
                    continue;
                }
                if y + y_offset >= self.numbers_board.len() as i16 || y + y_offset < 0 || x + x_offset >= self.numbers_board[0].len() as i16 || x + x_offset < 0  {
                    continue;
                }
                if self.flagged_board[(y+y_offset) as usize][(x+x_offset) as usize] == TileMarking::Mine {
                    c += 1;
                }
            }
        }
        if c == n {
            for y_offset in -1..=1 {
                for x_offset in -1..=1 {
                    if y_offset == 0 && x_offset == 0 {
                        continue;
                    }
                    if y + y_offset >= self.numbers_board.len() as i16 || y + y_offset < 0 || x + x_offset >= self.numbers_board[0].len() as i16 || x + x_offset < 0  {
                        continue;
                    }
                    // call open function here(?)
                }
            }
        }
        
    }


    fn question_and_bomb_marks(&mut self) {

    }


    fn closed_squares(&mut self) {

    }

    
    fn first_click(&mut self, y: usize, x: usize) {
        if self.numbers_board[y][x] >= 100 {
            for y in 0..self.numbers_board.len() {
                for x in 0..self.numbers_board[0].len() {
                    
                }
            }
        }
    }


    fn handle_click(&mut self, y: usize, x: usize, clicktype: u8) { // 0 is primary, 2 is rclick
        /*
        add enum for win or lose etc
        if rclick then run questions and bomb marks function. 
        (add mark array with mark enums)
        if lclick then either:
        0. Check if it's first click. If so, run first click function
        1. It's on a revealed, non-zero square:
            call chord to "chord" it
        2. It's on a not-opened square
            closed_squares handles it
        3. It's on an open square
            ignore it(?)
        more(?)
        */
    }

}

fn main() {
    let n = 10;
    let minecount = ((n*n) as f64 *0.2 ) as usize;
    let now = Instant::now();
    let mut m = MineSweeper::new(n, n, minecount);
    let num1 = 5;
    let num2 = 5;
    m.reveal(num1, num2);
    m.debugprint();
    println!("{}", now.elapsed().as_millis());
     
}