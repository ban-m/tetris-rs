#![feature(slice_patterns)]
extern crate rand;
extern crate termios;
mod tetris;
use std::thread;
use std::time;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use std::io::Read;
use std::io;
use tetris::Command;
use std::sync::mpsc;
fn main(){
    let (tx,rx) = mpsc::channel();
    thread::spawn(move ||{
        let stdin = 0;
        let termios = Termios::from_fd(stdin).unwrap();
        let mut new_termios = termios.clone();  // make a mutable copy of termios 
        // that we will modify
        new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
        tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();
        let mut reader = io::stdin();
        let mut buf1 = [0;1];
        loop{
            if let Ok(_) = reader.read_exact(&mut buf1){
                if let Some(command) = Command::parse_command(buf1[0]){
                    if let Err(_) =  tx.send(command){
                        break;
                    }
                    if let Command::Quite = command{
                        break;
                    }
                }
            }
        }
        tcsetattr(stdin, TCSANOW, & termios).unwrap();  // reset the stdin to
    });
    let mut tetris = tetris::Tetris::new(40,40);
    let sleep = 100;
    'main: loop{
        println!("{}",tetris);
        tetris = tetris.interval_update();
        let start = time::Instant::now();
        while let Ok(command) = rx.try_recv(){
            match command {
                Command::Down | Command::Immd | Command::MoveLeft |
                Command::MoveRight | Command::Reserve | Command::Rotate
                    => tetris = tetris.update(command),
                Command::Quite => break 'main,
                _ => {},
            };
            println!("{}",tetris);
        }
        let end = time::Instant::now();
        if end - start < time::Duration::from_millis(sleep){
            thread::sleep(time::Duration::from_millis(sleep) + (end- start));
        }
    }
}
