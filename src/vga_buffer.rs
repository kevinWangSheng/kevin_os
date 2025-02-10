
#[allow(dead_code)]
#[derive(Debug,Clone, Copy,PartialEq, Eq)]
#[repr(u8)]
enum Color{
    Black=0,
    Blue=1,
    Green=2,
    Cyan=3,
    Red=4,
    Magenta=5,
    Brown=6,
    LightGray=7,
    DarkGray=8,
    LightBlue=9,
    LightGreen=10,
    LightCyan=11,
    LightRed=12,
    Pink=13,
    Yellow=14,
    White=15,

}

/// the struct is like that :Bit layout of the color byte:
///  7  6  5  4  3  2  1  0
///  ├──┴──┴──┴──┴──┴──┴──┴──┤
///  │ bg color │ fg color   │
///  └──────────┴───────────-┘
#[derive(Debug,Clone, Copy,PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode{
    pub fn new(forceround:Color,background:Color)->Self{
        ColorCode(((background as u8) << 4) |(forceround as u8) )
    }
}

#[repr(C)]
#[derive(Debug,Clone, Copy,PartialEq, Eq)]
struct ScreenChar{
    ascii_character:u8,
    color_code:ColorCode
}
const BUFFER_HEIGHT:usize = 25;
const BUFFER_WIDTH:usize = 80;

/// 
#[repr(transparent)]
struct Buffer{
    chars:[[ScreenChar;BUFFER_WIDTH];BUFFER_HEIGHT]
}

pub struct Write{
    column_position:usize,
    color_code:ColorCode,
    buffer:&'static mut Buffer
}

impl Write{
    pub fn write_byte(&mut self,byte:u8){
        match byte {
            b'\n'=> self.new_line(),
            byte=>{
                if self.column_position>=BUFFER_WIDTH{
                    self.new_line();
                }
                let row = BUFFER_HEIGHT-1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col] = ScreenChar{
                    ascii_character:byte,
                    color_code
                };
                self.column_position+=1;
            }
        }
    }

    pub fn write_string(&mut self,s:&str){
        for byte in s.bytes(){
            match byte{
                0x20..=0x7e | b'\n'=>self.write_byte(byte),
                _=>self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&self){
        todo!()
    }
}

pub fn write_something(){
    let mut write = Write{
        column_position:0,
        color_code:ColorCode::new(Color::Brown, Color::Green),
        buffer:unsafe {
            &mut *(0xb8000 as *mut Buffer)
        }
    };
    write.write_byte(b'H');
    write.write_string("ello");
    write.write_string(" world!");
}