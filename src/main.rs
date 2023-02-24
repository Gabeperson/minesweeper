use dolphine::Browser;
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
use std::iter::zip;
use std::time::Instant;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use std::collections::HashMap;
use dolphine::Report;

static FILES: Dir = dolphine::include_dir!("web");
static MINESWEEPER: Lazy<Mutex<Minesweeper>> = Lazy::new(|| Mutex::new(Minesweeper::new(60, 60, 500)));
// todo bitwise shift for indications??? NOPE
// todo first click always safe
// if more than bombs num then reveal only squares that arent bombs
// timer(??)
// bombcount(??) 


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
struct Minesweeper {
    numbers_board: Vec<Vec<u8>>,
    revealed_board: Vec<Vec<bool>>,
    flagged_board: Vec<Vec<TileMarking>>,
    started: bool,
    mines: usize,
}

#[derive(Clone, Debug)]
enum GameResult {
    Continuing,
    Won,
    Lost,
}


impl Minesweeper {
    fn new(x: isize, y: isize, mines: usize) -> Minesweeper {
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
                numbers_board[y_value as usize][x_value as usize] += 100;
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
        
        Minesweeper {
            numbers_board,
            revealed_board: vec![vec![false; x as usize]; y as usize],
            flagged_board: vec![vec![TileMarking::None; x as usize]; y as usize],
            started: false,
            mines,
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
        println!("ENDLINE");
    }

    fn reveal(&mut self, y: usize, x: usize) -> Vec<(i16, i16, u8)> {
        let original_revealed_board = self.revealed_board.clone();
        let mut stack = Vec::new();
        if self.numbers_board[y][x] > 0 {
            self.revealed_board[y][x] = true;
            return vec![(y as i16, x as i16, self.numbers_board[y][x])]; // todo
        }
        if self.revealed_board[y][x] {
            return vec![];
        }
        stack.push((y as i16, x as i16, 0));
        self.reveal_impl(&mut stack, (self.numbers_board.len() as i16, self.numbers_board[0].len() as i16));

        let mut ret_vec = Vec::new();
        for y in 0..original_revealed_board.len() {
            for x in 0..original_revealed_board[0].len() {
                if original_revealed_board[y][x] == self.revealed_board[y][x] {
                    continue;
                }
                //dbg!("got here");
                ret_vec.push((y as i16, x as i16, self.numbers_board[y][x]))
            }
        }
        return ret_vec
    }
    
    fn reveal_impl(&mut self, stack: &mut Vec<(i16, i16, u8)>, sizes: (i16, i16)) { // very bad code involved. Too lazy to refactor ok bye :D
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

    fn chord(&mut self, y: i16, x: i16) -> (u8, Option<Vec<(i16, i16, u8)>>) {
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
        
        if c >= n {
            let mut ret_vec = Vec::new();
            for y_offset in -1..=1 {
                for x_offset in -1..=1 {
                    if y_offset == 0 && x_offset == 0 {
                        continue;
                    }
                    if y + y_offset >= self.numbers_board.len() as i16 || y + y_offset < 0 || x + x_offset >= self.numbers_board[0].len() as i16 || x + x_offset < 0  {
                        continue;
                    }
                    if !self.revealed_board[(y+y_offset) as usize][(x+x_offset) as usize] {
                        if self.numbers_board[(y+y_offset) as usize][(x+x_offset) as usize] >= 100 {
                            if self.flagged_board[(y+y_offset) as usize][(x+x_offset) as usize] != TileMarking::Mine {
                                return (2, Some(self.handle_loss()));
                            }
                            continue;
                        }
                        let r = self.reveal((y+y_offset) as usize, (x+x_offset) as usize);
                        ret_vec.extend_from_slice(&r);
                    }
                }
            }
            return (0, Some(ret_vec))
        }
        return (0, None)
    }


    fn question_and_bomb_marks(&mut self, y: usize, x: usize) -> u8 { // 0: bomb 1: question 2: removed
        match self.flagged_board[y][x] {
            TileMarking::Mine => {
                self.flagged_board[y][x] = TileMarking::Question;
                0
            },
            TileMarking::None => {
                self.flagged_board[y][x] = TileMarking::Mine;
                1
            }
            TileMarking::Question => {
                self.flagged_board[y][x] = TileMarking::None;
                2
            }
        }
    }

    
    fn first_click(&mut self, y: usize, x: usize) { // TODO all safe on first click...??
        self.started = true;
        if self.numbers_board[y][x] >= 100 {
            'blck: {
                for y1 in 0..self.numbers_board.len() {
                    for x1 in 0..self.numbers_board[0].len() {
                        if self.numbers_board[y1][x1] < 90 {
                            for y_o in -1..=1 {
                                for x_o in -1..=1 {
                                    if y1 as i16 + y_o >= self.numbers_board.len() as i16 || y1 as i16 + y_o < 0 || x1 as i16 + x_o >= self.numbers_board[0].len() as i16 || x1 as i16 + x_o < 0  {
                                        continue;
                                    }
                                    self.numbers_board[(y1 as i16+y_o) as usize][(x1 as i16+x_o) as usize] += 1;
                                }
                            }
                            self.numbers_board[y1][x1] += 100;
                            break 'blck
                        }
                    }
                }
            }
            for y_o in -1..=1 {
                for x_o in -1..=1 {
                    if y as i16 + y_o >= self.numbers_board.len() as i16 || y as i16 + y_o < 0 || x as i16 + x_o >= self.numbers_board[0].len() as i16 || x as i16 + x_o < 0  {
                        continue;
                    }
                    self.numbers_board[(y as i16 + y_o) as usize][(x as i16 + x_o) as usize] -= 1;
                    if y_o == 0 && x_o == 0 {
                        self.numbers_board[(y as i16 + y_o) as usize][(x as i16 + x_o) as usize] -= 100;
                    }
                }
            }
        }
    }

    fn handle_loss(&mut self) -> Vec<(i16, i16, u8)> {
        let mut v: Vec<(i16, i16, u8)> = vec![];
        for (indexy, y) in self.numbers_board.iter().enumerate() {
            for (indexx, x) in y.iter().enumerate() {
                if *x >= 100 {
                    v.push((indexy as i16, indexx as i16, 0));
                }
            }
        }
        
        return v;
    }


    fn handle_click(&mut self, y: usize, x: usize, clicktype: u8) -> (u8, Option<Vec<(i16, i16, u8)>>) {

        /*
        return values
        0 if ongoing
        1 if won
        2 if lost
        3 if sending marking
        
        */
        // clicktype: 0 is primary, 2 is rclick
        let ret;
        'blck: {
            match clicktype {
                0 => {
                    if self.flagged_board[y][x] != TileMarking::None {
                        return (200, None);
                    }
                    if self.started {
                        // if bomb, we lose
                        if self.numbers_board[y][x] >= 100 {
                            return (2, Some(self.handle_loss()));
                        }
                        if self.revealed_board[y][x] && self.numbers_board[y][x] != 0 {
                            ret = self.chord(y as i16, x as i16);
                            break 'blck;
                        } else {
                            let r = self.reveal(y, x);
                            ret = (0, Some(r));
                            break 'blck;
                        }
                    }
                    // if we haven't started
                    self.first_click(y, x);
                    let r = self.reveal(y, x);
                    return (0, Some(r));
                }
                2 => {
                    if self.revealed_board[y][x] {
                        return (200, None)
                    }
                    let r = self.question_and_bomb_marks(y, x);
                    /*
                    0: question
                    1: mine
                    2: none
                    */
                    return (3, Some(vec![(y as i16, x as i16, r)]));
                }
                _ => return (100, None),
            }
        }
        let mut num_mines = 0;
        for row in self.revealed_board.iter() {
            for boolean in row {
                if *boolean {
                    num_mines += 1;
                }
            }
        }
        if num_mines == self.mines {
            return (1, None);
        }
        ret

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

#[dolphine::async_function]
async fn start_game() -> Result<(), Report> {
    let mut minesweeper = MINESWEEPER.lock().await;
    *minesweeper = Minesweeper::new(60, 60, 500);
    println!("Made new minesweeper");
    Ok(())
}

#[dolphine::async_function]
async fn handler(y: usize, x: usize, click_type: usize) -> Result<(u8, Option<Vec<(i16, i16, u8)>>), Report> {
    let mut minesweeper = MINESWEEPER.lock().await;

    let resp = minesweeper.handle_click(y, x, click_type as u8);
    //dbg!(&resp);
    Ok(resp)
}

#[rocket::main]
async fn main() {
    let mut dolphine = Dolphine::new();
    dolphine.set_static_file_directory(&FILES);
    dolphine.register_function("send", handler, 3);
    dolphine.register_function("start", start_game, 0);
    dolphine.open_page(Browser::chrome());
    dolphine.init(true).await;
}