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
            revealed_board: vec![vec![false; x as usize]; y as usize],
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

    fn start_reveal_looparray(&mut self, y: usize, x: usize) -> usize {
        let mut ret = 0;
        let (maxy, maxx) = (self.revealed_board.len(), self.revealed_board[0].len());
        let (y, x) = (y as i16, x as i16);
        const CELL_8_ARRAY: &'static [(i16, i16, u8); 8] = &[
            (-1, -1, 0b1000_0000),
            (-1, 0, 0b0100_0000),
            (-1, 1, 0b0010_0000),
            (0, 1, 0b0001_0000),
            (1, 1, 0b0000_1000),
            (1, 0, 0b0000_0100),
            (1, -1, 0b0000_0010),
            (0, -1, 0b0000_0001),
        ];
        let mut arr: Vec<(i16, i16, u8)> = Vec::new(); 
        arr.push((y, x, 0));
        while arr.len() > 0 {
            if arr.len() > ret {
                ret = arr.len();
            }
            let mut temp_arr = Vec::new();
            for tup in arr.iter() {
                self.revealed_board[tup.0 as usize][tup.1 as usize] = true;
                for (index, (y_offset, x_offset, dir_bits)) in CELL_8_ARRAY.iter().enumerate() {
                    if tup.0 + y_offset >= maxy as i16 || tup.0 + y_offset < 0 || tup.1 + x_offset >= maxx as i16 || tup.1 + x_offset < 0{
                        //println!("oob");
                        continue; // out of bounds
                    }
                    if self.revealed_board[(tup.0+y_offset) as usize][(tup.1+x_offset) as usize] {
                        //println!("ar");
                        continue; // tile we are checking is already revealed
                    }
                    if self.numbers_board[(tup.0+y_offset) as usize][(tup.1+x_offset) as usize] > 0 {
                        // if there's number, no need to check anymore. We just set it
                        // to true and be done with it. All revealable tiles will be reached anyway. 
                        self.revealed_board[(tup.0+y_offset) as usize][(tup.1+x_offset) as usize] = true;
                        //println!("alrnum");
                        continue;
                    }
                    temp_arr.push((tup.0+y_offset, tup.1+x_offset, 0b0000_0000));
                    //println!("Extended array");

                }
            }
            temp_arr.sort();
            temp_arr.dedup();
            arr.clear();
            arr.extend_from_slice(&temp_arr);
            //println!("{:?}", &arr);
        }
        return ret;
    }

    fn start_reveal(&mut self, y: usize, x: usize) -> usize {
        // REMOVE LATER
        let mut c = 1;
        let mut stack = Vec::new();
        if self.numbers_board[y][x] > 0 {
            self.revealed_board[y][x] = true;
            return 0;
        }
        if self.revealed_board[y][x] {
            return 0;
        }
        stack.push((y as i16, x as i16, 0));
        // REMOVE C
        self.reveal(&mut stack, (self.numbers_board.len() as i16, self.numbers_board[0].len() as i16), &mut c);
        // REMOVE
        return c;
    }

    fn reveal_easiest(&mut self, y: usize, x: usize) -> usize {
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
        self.revealed_board[y][x] = true;
        let mut changed = true;
        let mut times = 0;
        while changed {
            times += 1;
            changed = false;
            for y_coord in 0..self.numbers_board.len() as i16 {
                for x_coord in 0..self.numbers_board[0].len() as i16 {
                    if self.numbers_board[y_coord as usize][x_coord as usize] != 0 || !self.revealed_board[y_coord as usize][x_coord as usize]{
                        continue;
                    }

                    for (y_offset, x_offset) in CELL_8_ARRAY {
                        if y_coord + y_offset >= self.numbers_board.len() as i16 || y_coord + y_offset < 0 || x_coord + x_offset >= self.numbers_board[0].len() as i16 || x_coord + x_offset < 0 {
                            continue;
                        }
                        if !self.revealed_board[(y_coord+y_offset) as usize][(x_coord+x_offset) as usize] {
                            self.revealed_board[(y_coord+y_offset) as usize][(x_coord+x_offset) as usize] = true;
                            changed = true;
                        }
                    }
                }
            }
        }
        return times;
    }
    
    // REMOVE C
    fn reveal(&mut self, stack: &mut Vec<(i16, i16, u8)>, sizes: (i16, i16), l: &mut usize) {
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

            // REMOVE
            if stack.len() > *l {
                *l = stack.len()
            }
            
        }

    }


    fn reveal1(&mut self, cell: (usize, usize), stack: &mut Vec<()>) {
        if self.revealed_board[cell.0][cell.1] == true {
            return
        }
        self.revealed_board[cell.0][cell.1] = true;
        let (xmin, xmax, ymin, ymax) = (0, self.numbers_board[0].len(), 0, self.numbers_board.len());
        if self.numbers_board[cell.0][cell.1] != 0 {
            return;
        }
        // the square is a zero
        for y in -1..=1 {
            for x in -1..=1 {
                //println!("y: {}, x: {}", y+y_cell, x+x_cell);
                if y+(cell.0 as isize) >= ymax as isize || y+(cell.0 as isize) < ymin || x + (cell.1 as isize) >= xmax as isize || x+(cell.1 as isize) < xmin {
                    continue;
                }
                if y == 0 && x == 0 {
                    continue;
                }
                //println!("got hereee");
                self.reveal1(((y+(cell.0 as isize)) as usize, (x+(cell.1 as isize)) as usize), stack);
            }
        }
    }

}

fn main() {
    let n = 1000;
    //m.debugprint();
    let mut buf = String::new();
    let mut buf2 = String::new();
    //println!("Y value");
    io::stdin()
          .read_line(&mut buf)
          .unwrap();
    //println!("X value");
    io::stdin()
          .read_line(&mut buf2)
          .unwrap();

    buf = buf.trim().to_string();
    buf2 = buf2.trim().to_string();
    //println!("{}, {}", &buf, &buf2);   


    let mut m = MineSweeper::new(n, n, 0);
    let now = Instant::now();
    let x = m.start_reveal_looparray(buf.parse().unwrap(), buf2.parse().unwrap());
    //m.start_reveal_looparray(buf.parse().unwrap(), buf2.parse().unwrap());
    //m.debugprint();
    println!("{}, {}", x, now.elapsed().as_millis());


    let mut m = MineSweeper::new(n, n, 0);
    let now = Instant::now();
    let x = m.start_reveal(buf.parse().unwrap(), buf2.parse().unwrap());
    //m.debugprint();
    println!("{}, {}", x, now.elapsed().as_millis());

    let mut m = MineSweeper::new(n, n, 0);
    let now = Instant::now();
    let x = m.reveal_easiest(buf.parse().unwrap(), buf2.parse().unwrap());
    //m.debugprint();
    println!("{}, {}", x, now.elapsed().as_millis());
     
}