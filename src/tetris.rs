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
    board:Vec<Vec<Output>>,
    user_status:User,
}

impl fmt::Display for Tetris{
    fn fmt(&self,f:&mut fmt::Formatter)-> fmt::Result{
        // clear the screen
        writeln!(f,"\x1b[1;1H")?;
        writeln!(f,"\x1b[J")?;
        // print various desctiption
        writeln!(f,"width:{},heigth:{},next:{},reserve:{},current:({},{}),board:{},user:{}",
                 self.width,self.height,self.next,self.reserve,
                 (self.current).0,(self.current).1,(self.board).len(),self.user_status)?;
        let board = self.board.clone();
        let current:Vec<_> = self.current.0.get_location_by(&self.current.1).into_iter()
            .map(|(x,y,_)|(x,y,Output::with_shape('#','b'))).collect();
        let board = Tetris::add(board,&current);
        for line in board{
            for element in line.iter(){
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
        writeln!(f,"occupied cell:{}",
                 self.board.iter().flat_map(|e|e.iter()).filter(|e|e.not_default()).count())?;
        Ok(())
    }
}

impl Tetris{
    pub fn new(width:usize,height:usize)->Self{
        let next = Block::new();
        let reserve = Block::new();
        let current = (Block::T,Position::create_at(width/2,3));
        let user_status = User::new();
        let board = vec![vec![Output::new();width];height];
        Tetris{width,height,next,reserve,current,board,user_status}
    }
    pub fn interval_update(self)->Self{
        self.apply(Command::Down).unwrap_or_else(|e|e.next_block()).resolve()
    }
    fn next_block(self)->Self{
        let position = Position::create_at(self.width/2,3);
        let block = self.next;
        let next = Block::new();
        let board = Tetris::add(self.board,&self.current.0.get_location_by(&self.current.1));
        Tetris{current:(block,position),next:next,board:board,..self}
    }
    fn resolve(self)->Self{
        let board:Vec<_> = self.board.iter()
            .filter_map(|e|Tetris::line_resolve(e,self.width)).collect();
        let mut padding = vec![vec![Output::new();self.width];self.height - board.len()];
        if padding.len() > 0{
            padding.extend(board);
            Tetris{board:padding,..self}
        }else{
            Tetris{board:board,..self}
        }
    }
    fn line_resolve(line:&[Output],width:usize)->Option<Vec<Output>>{
        if line.iter().skip(3).take(width-6).all(|e|e.not_default()){
            None
        }else{
            Some(line.iter().map(|&e|e).collect())
        }
    }
    fn add(mut board:Vec<Vec<Output>>,location:&[(usize,usize,Output)])->Vec<Vec<Output>>{
        for &(x,y,output) in location{
            board[y][x] = output;
        }
        board
    }
    pub fn update(self,command:Command)->Self{
        self.apply(command).unwrap_or_else(|e|e).resolve()
    }
    fn in_range(&self,location:&[(usize,usize,Output)])->bool{
        location.iter()
            .all(|&(x,y,_)|self.width > x + 2 && self.height > y + 2 && x > 2 )
    }
    fn not_collide(&self,location:&[(usize,usize,Output)])->bool{
        location.iter()
            .all(|&(x,y,_)|!self.does_collide(x,y))
    }
    fn does_collide(&self,x:usize,y:usize)->bool{
        (0..self.height).flat_map(|y|(0..self.width).map(move |x|(x,y)))
            .filter(|&(x,y)|self.board[y][x].not_default())
            .any(|(s,t)| x == s && y == t)
    }
    fn apply(mut self,command:Command)->Result<Self,Self>{
        match command {
            Command::Immd => Ok(self.immediatly()),
            Command::Rotate => {
                let (block,position) = self.current;
                let position = position.rotate();
                let result:Vec<_> = block.get_location_by(&position);
                if self.in_range(&result) && self.not_collide(&result){
                    self.current = (block,position);
                    Ok(self)
                }else{
                    Err(self)
                }
            },
            Command::MoveRight | Command::MoveLeft | Command::Down => 
                if self.is_valid_after(&command){
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
            _ => unreachable!(),
        }
    }
    fn immediatly(self)->Self{
        self
    }
    fn is_valid_after(&self,command:&Command)->bool{
        let position = self.current.1.move_by(command);
        let positions = self.current.0.get_location_by(&position);
        self.in_range(&positions) && self.not_collide(&positions)
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
    NoOp,
    Quite,
}

impl Command{
    pub fn parse_command(buffer:u8)->Option<Self>{
        match buffer{
            119 => Some(Command::Immd),
            115 => Some(Command::Down),
            100 => Some(Command::MoveRight),
            97 => Some(Command::MoveLeft),
            113 => Some(Command::Quite),
            106 => Some(Command::Rotate),
            107 => Some(Command::Reserve),
            _ => Some(Command::NoOp),
        }
    }
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
    fn new()->Self{
        let mut rng = thread_rng();
        *rng.choose(&BLOCKS).unwrap_or(&Block::I)
    }
    fn get_location_by(&self,position:&Position)->Vec<(usize,usize,Output)>{
        use self::Block::*;
        use self::Angle::*;
        let angle = position.angle;
        let x = position.x;
        let y = position.y;
        let create = |x,y| (x,y,Output::with_shape('O','b'));
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
    fn not_default(&self)->bool{
        self.shape != ' '  || self.color != ' '
    }
           
}
impl fmt::Display for Output{
    fn fmt(&self,f:&mut fmt::Formatter)->fmt::Result{
        if self.not_default(){
            write!(f,"\x1b[41m{}\x1b[m",self.shape)
        }else{
            write!(f,"{}",self.shape)
        }
    }
}
