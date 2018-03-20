use rand::{Rng,thread_rng};
use std::fmt;
const BLOCKS:[Block;7] = [Block::I,Block::J,Block::L,Block::O,
                        Block::S,Block::T,Block::Z];
const ANGLES:[Angle;4] = [Angle::Up,Angle::Down,Angle::Right,Angle::Left];
pub struct Tetris{
    width:usize,
    height:usize,
    next:Block,
    reserve:Block,
    current:(Block,Position),
    board:Vec<(Block,Position)>,
    user_status:User,
}

impl fmt::Display for Tetris{
    fn fmt(&self,f:&mut fmt::Formatter)-> fmt::Result{
        // clear the screen
        println!("\x1b[1;1H");
        println!("\x1b[J");
        // print various desctiption
        writeln!(f,"width:{},heigth:{},next:{},reserve:{},current:({},{}),board:{},user:{}",
                 self.width,self.height,self.next,self.reserve,
                 (self.current).0,(self.current).1,(self.board).len(),self.user_status)?;
        let board = self.create_board();
        for line in board{
            for element in line{
                write!(f,"{}",element)?;
            }
            writeln!(f,"")?;
        }
        Ok(())
    }
}
impl fmt::Debug for Tetris{
    fn fmt(&self,f:&mut fmt::Formatter)->fmt::Result{
        writeln!(f,"width:{},height{},next:{},reserve:{},current:({},{}),board:{},user:{}",
                 self.width,self.height,self.next,self.reserve,
                 (self.current).0,(self.current).1,(self.board).len(),self.user_status)?;
        for &(ref b,ref p) in &self.board{
            writeln!(f,"{},{:?}",b,p)?;
        }
        Ok(())
    }
}

impl Tetris{
    pub fn new(width:usize,height:usize)->Self{
        let next = Block::new();
        let reserve = Block::new();
        let current = (Block::T,Position::create_at(width/2,3));
        let user_status = User::new();
        let board = vec![];
        Tetris{width,height,next,reserve,current,board,user_status}
    }
    pub fn tic(self)->Self{
        let (block,position) = self.current;
        let new_position = position.tic();
        let result:Vec<_> = block
            .update_location(&new_position)
            .into_iter().map(|(x,y,_)|(x,y)).collect();
        let in_range = result.iter()
            .all(|&(x,y)|x < self.width  && y < self.height);
        let not_collide = result.iter()
            .all(|&(x,y)|!self.does_collide(x,y));
        if in_range && not_collide{
            // keep current block
            Tetris{current:(block,new_position),..self}
        }else{
            // Already settled. Next block.
            let mut board = self.board;
            board.push(self.current);
            let position = Position::create_at(self.width/2,3);
            let block = self.next;
            let next = Block::new();
            Tetris{current:(block,position),next:next,board:board,..self}
        }
    }
    pub fn update(self,command:Command)->Self{
        self.apply(command).unwrap_or_else(|e|e)
    }
    fn apply(mut self,command:Command)->Result<Self,Self>{
        match command {
            Command::Immd => Ok(self.immediatly()),
            Command::Rotate => {
                let (block,position) = self.current;
                let position = position.rotate();
                let result:Vec<_> = block
                    .update_location(&position)
                    .into_iter().map(|(x,y,_)|(x,y))
                    .collect();
                let in_range = result.iter()
                    .all(|&(x,y)|self.width > x && self.height > y);
                let not_collide = result.iter()
                    .all(|&(x,y)|!self.does_collide(x,y));
                if in_range && not_collide{                    
                    self.current = (block,position);
                    Ok(self)
                }else{
                    Err(self)
                }
            },
            Command::MoveRight | Command::MoveLeft | Command::Down => 
                if self.is_valid(&command){
                    let (block,position) = self.current;
                    let position = position.move_by(&command);
                    self.current = (block,position);
                    Ok(self)
                }else{
                    Err(self)
                },
            Command::Reserve => {
                let (current,position) = self.current;
                self.current = (self.reserve,position);
                self.reserve = current;
                Ok(self)
            }
        }
    }
    fn immediatly(self)->Self{
        self
    }
    fn does_collide(&self,x:usize,y:usize)->bool{
        self.board.iter()
            .flat_map(|e|(e.0).update_location(&e.1).into_iter())
            .map(|(x,y,_)|(x,y))
            .any(|(s,t)| x == s && y == t)
    }
    fn is_valid(&self,command:&Command)->bool{
        let position = self.current.1.move_by(command);
        let positions = self.current.0.update_location(&position);
        let positions:Vec<_> = positions.into_iter().map(|e|(e.0,e.1)).collect();
        let not_collide = positions.iter()
            .all(|&(x,y)|!self.does_collide(x,y));
        let in_range = positions.iter().all(|&(x,y)| x < self.width && 0 < x && y < self.height);
        in_range && not_collide
    }
    fn new_block(&self)-> (Block,Position){
        let position = Position::create_at(self.width/2,self.height);
        let block = Block::new();
        (block,position)
    }
    fn create_board(&self)->Vec<Vec<Output>>{
        self.board.iter()
            .chain([self.current.clone()].iter())
            .fold(vec![vec![Output::new();self.width];self.height],
                  |acc,&(ref block,position)|block.update(&position,acc))
    }
}
#[derive(Debug,Clone,Copy)]
pub enum Command{
    Rotate,
    Immd,
    MoveRight,
    MoveLeft,
    Down,
    Reserve,
}

#[derive(Copy,Clone)]
enum Block{
    I,
    J,
    L,
    O,
    S,
    T,
    Z
}

impl Block{
    fn update(&self,position:&Position,mut board:Vec<Vec<Output>>)->Vec<Vec<Output>>{
        for (x,y,output) in self.update_location(position){
            board[y][x] = output;
        }
        board
    }
    fn new()->Self{
        let mut rng = thread_rng();
        *rng.choose(&BLOCKS).unwrap_or(&Block::I)
    }
    fn update_location(&self,position:&Position)->Vec<(usize,usize,Output)>{
        use self::Block::*;
        use self::Angle::*;
        let angle = position.angle;
        let x = position.x;
        let y = position.y;
        let create = |x,y| (x,y,Output{shape:'#',color:'b'});
        match (*self,angle){
            (I,Up)   => vec![create(x,y),create(x,y+1),create(x,y-1),create(x,y-2)],
            (I,Down) => vec![create(x,y),create(x,y-1),create(x,y+1),create(x,y+2)],
            (I,Right)=> vec![create(x,y),create(x-1,y),create(x+1,y),create(x+2,y)],
            (I,Left) => vec![create(x,y),create(x+1,y),create(x-1,y),create(x-2,y)],
            (J,Up)   => vec![create(x,y),create(x,y-1),create(x,y-2),create(x-1,y)],
            (J,Down) => vec![create(x,y),create(x+1,y),create(x,y+1),create(x,y+2)],
            (J,Right)=> vec![create(x,y),create(x,y-1),create(x+1,y),create(x+2,y)],
            (J,Left) => vec![create(x,y),create(x,y+1),create(x-1,y),create(x-2,y)],
            (L,Up)   => vec![create(x,y),create(x,y-1),create(x,y-2),create(x+1,y)],
            (L,Down) => vec![create(x,y),create(x-1,y),create(x,y+1),create(x,y+2)],
            (L,Right)=> vec![create(x,y),create(x,y+1),create(x+1,y),create(x+2,y)],
            (L,Left) => vec![create(x,y),create(x,y-1),create(x-1,y),create(x-2,y)],
            (O,Up)   => vec![create(x,y),create(x+1,y),create(x,y-1),create(x+1,y-1)],
            (O,Down) => vec![create(x,y),create(x-1,y),create(x,y+1),create(x-1,y+1)],
            (O,Right)=> vec![create(x,y),create(x,y+1),create(x+1,y),create(x+1,y+1)],
            (O,Left) => vec![create(x,y),create(x-1,y),create(x,y-1),create(x-1,y-1)],
            (Z,Up)   => vec![create(x,y),create(x-1,y-1),create(x,y-1),create(x+1,y)],
            (Z,Down) => vec![create(x,y),create(x-1,y),create(x,y+1),create(x+1,y+1)],
            (Z,Right)=> vec![create(x,y),create(x,y+1),create(x+1,y),create(x+1,y-1)],
            (Z,Left) => vec![create(x,y),create(x,y-1),create(x-1,y),create(x-1,y+1)],
            (S,Up)   => vec![create(x,y),create(x+1,y-1),create(x,y-1),create(x-1,y)],
            (S,Down) => vec![create(x,y),create(x+1,y),create(x,y+1),create(x-1,y+1)],
            (S,Right)=> vec![create(x,y),create(x,y-1),create(x+1,y),create(x+1,y+1)],
            (S,Left) => vec![create(x,y),create(x,y+1),create(x-1,y),create(x-1,y-1)],
            (T,Up)   => vec![create(x,y),create(x-1,y),create(x+1,y),create(x,y+1)],
            (T,Down) => vec![create(x,y),create(x-1,y),create(x+1,y),create(x,y-1)],
            (T,Right)=> vec![create(x,y),create(x-1,y),create(x,y+1),create(x,y-1)],
            (T,Left) => vec![create(x,y),create(x+1,y),create(x,y+1),create(x,y-1)],
        }
    }
}

impl fmt::Display for Block{
    fn fmt(&self,f:&mut fmt::Formatter)-> fmt::Result{
        let output = match *self{
            Block::I => "I",
            Block::J => "J",
            Block::L => "L",
            Block::O => "O",
            Block::S => "S",
            Block::T => "T",
            Block::Z => "Z",
        };
        write!(f,"{}",output)
    }
}
#[derive(Clone,Copy)]
struct Position{
    x:usize,
    y:usize,
    angle:Angle
}
impl Position{
    fn new()->Self{
        Position{y:10,x:10,angle:Angle::Up}
    }
    fn create_at(x:usize,y:usize)->Self{
        let mut rng = thread_rng();
        let angle = rng.choose(&ANGLES).unwrap_or(&Angle::Up);
        Position{x,y,angle:*angle}
    }
    fn rotate(self)->Self{
        Position{angle:self.angle.rotate(),..self}
    }
    fn move_by(self,command:&Command)->Self{
        match *command{
            Command::Down => Position{y:self.y+1,..self},
            Command::MoveRight => Position{x:self.x+1,..self},
            Command::MoveLeft => Position{x:self.x-1,..self},
            _ => unreachable!(),
        }
    }
    fn tic(&self)->Self{
        Position{y:self.y+1,..*self}
    }
}
impl fmt::Display for Position{
    fn fmt(&self,f:&mut fmt::Formatter)-> fmt::Result{
        write!(f,"({},{},{})",self.x,self.y,self.angle)
    }
}

impl fmt::Debug for Position{
    fn fmt(&self,f:&mut fmt::Formatter)-> fmt::Result{
        write!(f,"({},{},{})",self.x,self.y,self.angle)
    }
}
#[derive(Debug,Clone,Copy)]
enum Angle{
    Up,
    Down,
    Right,
    Left,
}
impl Angle{
    fn rotate(self)->Self{
        match self{
            Angle::Up => Angle::Right,
            Angle::Down => Angle::Left,
            Angle::Right => Angle::Down,
            Angle::Left => Angle::Up,
        }
    }
}
impl fmt::Display for Angle{
    fn fmt(&self,f:&mut fmt::Formatter)-> fmt::Result{
        let output = match *self{
            Angle::Up => "Up",
            Angle::Down => "Down",
            Angle::Right => "Right",
            Angle::Left => "Left",
        };
        write!(f,"{}",output)
    }
}
#[derive(Debug)]
struct User{
    score:usize
}
impl User{
    fn new()->Self{
        User{score:0}
    }
}
impl fmt::Display for User{
    fn fmt(&self,f:&mut fmt::Formatter)->fmt::Result{
        write!(f,"{}",self.score)
    }
}

#[derive(Debug,Copy,Clone)]
struct Output{
    shape:char,
    color:char,
}
impl Output{
    fn new()->Self{
        Output{shape:' ',color:' '}
    }
    fn with_shape(shape:char,color:char)->Self{
        Output{shape,color}
    }
}
impl fmt::Display for Output{
    fn fmt(&self,f:&mut fmt::Formatter)->fmt::Result{
        write!(f,"{}",self.shape)
    }
}
