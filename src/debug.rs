extern crate rand;
mod tetris;
use std::thread;
use std::time;
use rand::{Rng,thread_rng};
const INTERVAL:u32= 1000_000_000;
use tetris::Command;
fn main(){
    //let choice = [Command::Down,Command::MoveLeft,Command::MoveRight,Command::Reserve,Command::Rotate];
    let choice = [Command::Rotate];
    let mut tetris = tetris::Tetris::new(40,40);
    let mut rng = thread_rng();
    for _ in 0 .. 10{
        thread::sleep(time::Duration::new(0,INTERVAL));
        tetris = tetris.tic();
        let command = rng.choose(&choice).unwrap_or(&Command::Down);
        tetris = tetris.update(*command);
        println!("{}",tetris);
    }
}
