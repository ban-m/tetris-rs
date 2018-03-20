#![feature(slice_patterns)]
extern crate libc;
mod tetris;
use std::{thread,time};
use std::io::Write;
use std::time::SystemTime;
use tetris;
const FPS:u32 = 1;
fn main() {
    println!("Hello, world!");
    let mut saved_termattr = libc::termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_cc: [0u8; 20],
        c_ispeed: 0,
        c_ospeed: 0,
    };
    unsafe {
        let mut ptr = &mut saved_termattr;
        libc::tcgetattr(0, ptr);
    }
    let mut termattr = saved_termattr;
    termattr.c_lflag = termattr.c_lflag & !(libc::ICANON | libc::ECHO);
    termattr.c_cc[libc::VMIN] = 1;
    termattr.c_cc[libc::VTIME] = 0;
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &termattr);
    }
    unsafe {
        libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK);
    }
    let mut input: [libc::c_char; 3] = [0; 3];
    let ptr = &mut input;
    let args:Vec<_> = std::env::args().collect();
    let throttle = time::Duration::new(0,1_000_000_000/FPS);
    let tetris = tetris::Tetris::new(80,120);
    let mut current_time = time::Instant::now();
    loop {
        let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 3) };
        if r > 0 {
            use tetris::Command;
            let command = match *ptr {
                [27, 91, 65] => Command::Immd,
                [27, 91, 66] => Command::Down,
                [27, 91, 67] => Command::MoveRight,
                [27, 91, 68] => Command::MoveLeft,
                [122, _, _] => break ;
                [120, _, _] => Command::Rotate,
                [99, _, _] => Command::Preserve,
                _ => println!("something else:{:?}",*ptr),
            };
        }
        thread::sleep(time::Duration::from_millis(300));
        std::io::stdout().flush().unwrap();
        if time::now() - current_time > throttle{
            current_time = time::now();
            println!("{}",tetris);
        }
    }
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &saved_termattr);
    }
}
