use std::io;
use std::io::prelude::*;
//use std::io::Read;
use std::fs::File;
use std::env;
//use std::io::fs::PathExtensions;
//use std::os;
extern crate rand;
use rand::{weak_rng, Rng};
extern crate sdl2;
//for benchmarking
extern crate time;
use time::PreciseTime;
use time::Duration;

//for delay Timer
use std::thread;

//for main event handling
use sdl2::event::Event;


//for display
use sdl2::video;
use sdl2::surface;
use sdl2::pixels;
use sdl2::rect::Rect;
static scale: u32 = 20;

//for keypad
use sdl2::keyboard::Keycode;

//too lazy to retype, ripped from https://github.com/mikezaby/chip-8.rs/blob/master/src/cpu.rs
static fontset: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80];


struct CHIP8<'a>{
    v: [u8; 16],
    I: u16,
    delay: u8, sound: u8,
    sp: u8, pc: u16,
    memory : [u8; 0xFFF],
    stack : [u16; 16],
    screen : [[u8; 64]; 32],
    draw_flag: bool,
    //eventually pull out into display mod
    window: surface::Surface<'a>,
    //eventually move to keypad mod
    keys : [bool; 16],
    keyWait : bool,
    keyV : usize,
    rng: rand::XorShiftRng
}

impl<'a> CHIP8<'a> {
    fn new() -> CHIP8<'a> {
        let mut ch = CHIP8 {
            v: [0x00; 16],
            I: 0x0000,
            delay: 0x00, sound: 0x00,
            sp: 0x00, pc: 0x0200,
            memory: [0; 0xFFF],
            stack: [0x0000; 16],
            screen: [[0x00; 64]; 32],
            draw_flag : true,
            //all credits for how to do this found at https://github.com/mikezaby/chip-8.rs/
            //window: video::set_video_mode(64*scale, 32*scale, 8,
            //                              &[video::SurfaceFlag::HWSurface],
            //                              &[video::VideoFlag::DoubleBuf]).unwrap(),
            window: surface::Surface::new(64*scale, 32*scale, pixels::PixelFormatEnum::RGB24).unwrap(),
            keys: [false; 16],
            keyWait: false,
            keyV: 0,
            rng: weak_rng()
        };
        for i in 0..80 {
            ch.memory[i] = fontset[i];
        }
        //for i in 0..1000 {
        //    Vec.push(rand::random<u8>())
        //}
        //for y in 0..32 {
        //    for x in 0..64 {
        //        if x % 3 == 0 {
        //            ch.screen[y][x] = 1;
        //        }
        //    }
        //}
        ch
    }
    fn load(&mut self, fileName: String){
        let mut filepath = env::current_dir().unwrap();
        filepath.push(fileName.trim());
        let mut reader = File::open(&mut filepath).unwrap(); //{
        //    Ok(reader) => reader,
        //    Err(_) => println!("Could not open file"),
        //};
        let mut buffer = Vec::new();

        reader.read_to_end(&mut buffer).unwrap();
        //let byteIt = reader.bytes();
        //'read : loop {
            for byte in buffer {
                //println!("{:X}", byte);
                self.memory[self.pc as usize] = byte;
                self.pc += 1;
            }
            self.pc = 0x0200;
        //}
    }

    fn display(&mut self, renderer: &mut sdl2::render::Renderer){
        //for debugging screen matrix values
        //for col in 0..32 {
        //    for bit in 0..64{
        //        print!("{:?}", self.screen[col][bit]);
        //    }
        //    print!("\n", );
        //}
        //ripped from https://github.com/mikezaby/chip-8.rs/
        if !self.draw_flag {return}
        let mut pixel: u8;
        let sc = scale as u32;
        //let pt = |&: p: usize| { (p as i16) * (scale as i16) };
        let pt = |p: usize| { (p as i32) * (scale as i32) };

        for y in 0..32 {
            for x in 0..64 {
                pixel = if self.screen[y][x] != 0 { 255 } else { 0 };
                renderer.set_draw_color(pixels::Color::RGB(pixel, pixel, pixel));
                renderer.fill_rect(Rect::new(pt(x), pt(y), sc, sc));
            }
        }
        renderer.present();
        //self.window.flip();
        //self.v[15] = 0;
    }

    //ripped from https://github.com/mikezaby/chip-8.rs/
    //modified to work with array of bools
    fn press(&mut self, key: Keycode, state: bool) {
        if self.keyWait {self.v[self.keyV] = key as u8; self.keyWait = false;}
        //println!("Keycode: {:?} State: {:?}", key, state);
    match key {
      Keycode::Num1 => self.keys[0x1] = state,
      Keycode::Num2 => self.keys[0x2] = state,
      Keycode::Num3 => self.keys[0x3] = state,
      Keycode::Num4 => self.keys[0xc] = state,
      Keycode::Q    => self.keys[0x4] = state,
      Keycode::W    => self.keys[0x5] = state,
      Keycode::E    => self.keys[0x6] = state,
      Keycode::R    => self.keys[0xd] = state,
      Keycode::A    => self.keys[0x7] = state,
      Keycode::S    => self.keys[0x8] = state,
      Keycode::D    => self.keys[0x9] = state,
      Keycode::F    => self.keys[0xe] = state,
      Keycode::Z    => self.keys[0xa] = state,
      Keycode::X    => self.keys[0x0] = state,
      Keycode::C    => self.keys[0xb] = state,
      Keycode::V    => self.keys[0xf] = state,
      _         => ()
    }
  }

    fn fetch(&mut self) -> u16{
        let mut opcode: u16 = self.memory[self.pc as usize] as u16;
        self.pc += 1;
        opcode = opcode.rotate_left(8);
        opcode |= self.memory[self.pc as usize] as u16;
        self.pc += 1;
        opcode
    }
    fn readByte(&mut self) -> u8 {
        self.pc += 1;
        self.memory[(self.pc -1) as usize]
    }
    fn execute(&mut self, opcode: u16){
        //println!("Program Counter: {:X}", self.pc);
        //println!("Opcode: {:X}", opcode);
        //thread::sleep_ms(5);
        if self.delay > 0 { self.delay -= 1;}
        if self.keyWait {return}
        match opcode {
            0x00E0 => {
                self.screen = [[0x00; 64]; 32];
                //should set draw flag?
                //self.v[15] = 1;
            },
            0x00EE => {
                self.pc = self.stack[(self.sp-1) as usize];
                self.sp -= 1;
            },
            add @ 0x1000 ... 0x1FFF=> {
                self.pc = add & 0x0FFF;
            },
            add @ 0x2000 ... 0x2FFF => {
                 self.stack[self.sp as usize] = self.pc;
                 self.pc = add & 0x0FFF;
                 self.sp += 1;
             },
            add @ 0x3000 ... 0x3FFF => {
                let x: u8 = (add & 0x0F00).rotate_right(8) as u8;
                let kk: u8 = (add & 0x00FF) as u8;
                if self.v[x as usize] == kk {
                    self.pc += 2;
                }
            },
            add @ 0x4000 ... 0x4FFF => {
                let x: usize = ((add & 0x0F00) >> 8) as usize;
                let kk: u8 = (add & 0x00FF) as u8;
                if self.v[x] != kk {
                    self.pc += 2;
                }
            },
            add @ 0x5000 ... 0x5FFF => {
                let x: u8 = (add & 0x0F00).rotate_right(8) as u8;
                let y:u8 = (add & 0x00F0).rotate_right(4) as u8;
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            },
            add @ 0x6000 ... 0x6FFF => {
                let x: u8 = (add & 0x0F00).rotate_right(8) as u8;
                let kk: u8 = (add & 0x00FF) as u8;
                self.v[x as usize] = kk;
            },
            add @ 0x7000 ... 0x7FFF => {
                let x: u8 = (add & 0x0F00).rotate_right(8) as u8;
                let kk: u8 = (add & 0x00FF) as u8;
                self.v[x as usize] = self.v[x as usize].wrapping_add(kk);
            },
            add @ 0x8000 ... 0x8FFF => {
                let x: u8 = ((add & 0x0F00) >> 8) as u8;
                let y:u8 = ((add & 0x00F0) >> 4) as u8;
                let z: u8 = (add & 0x000F) as u8;
                let f = 15;
                match z {
                    0 => {self.v[x as usize] = self.v[y as usize];},
                    1 => {self.v[x as usize] |= self.v[y as usize];},
                    2 => {self.v[x as usize] &= self.v[y as usize];},
                    3 => {self.v[x as usize] ^= self.v[y as usize];},
                    4 => {
                        let sX = self.v[x as usize];
                        self.v[x as usize] = self.v[x as usize].wrapping_add(self.v[y as usize]);
                        if self.v[x as usize] < sX {
                            self.v[f] = 1;
                        }
                        else {
                            self.v[f] = 0;
                        }
                    },
                    5 => {
                        if self.v[x as usize] > self.v[y as usize] {
                            self.v[f] = 1;
                        }
                        else {
                            self.v[f] = 0;
                        }
                        self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
                    },
                    6 => {
                        self.v[f] = self.v[x as usize] & 0x01;
                        self.v[x as usize] /= 2;
                    },
                    7 => {
                        if self.v[y as usize] > self.v[x as usize] {
                            self.v[f] = 1;
                        }
                        else {self.v[f] = 0;}
                        self.v[x as usize] = self.v[y as usize] - self.v[x as usize];
                    },
                    14 => {
                        self.v[f] = (self.v[x as usize] & 0x80) >> 7;
                        self.v[x as usize] *= 2;
                    },
                    _ => println!("Unexpexted opcode!!"),
                    }
                },
                add @ 0x9000 ... 0x9FF0 => {
                    let x: u8 = (add & 0x0F00).rotate_right(8) as u8;
                    let y:u8 = (add & 0x00F0).rotate_right(4) as u8;
                    if self.v[x as usize] != self.v[y as usize] {
                        self.pc += 2;
                    }
                },
                add @ 0xA000 ... 0xAFFF => {
                    self.I = 0x0FFF & add;
                },
                add @ 0xB000 ... 0xBFFF => {
                    self.pc = (0x0FFF & add).wrapping_add(self.v[0] as u16);
                },
                add @ 0xC000 ... 0xCFFF => {
                    let x: u8 = (add & 0x0F00).rotate_right(8) as u8;
                    let kk: u8 = (add & 0x00FF) as u8;
                    self.v[x as usize] = self.rng.gen::<u8>() & kk;
                },
                add @ 0xD000 ... 0xDFFF => {
                    let x: usize = ((add & 0x0F00) >> 8) as usize;
                    let y: usize = ((add & 0x00F0) >> 4) as usize;
                    let n:u8 = (add & 0x000F) as u8;
                    self.draw_flag = true;
                    //new way doing it by pixel
                    //for n bytes
                    self.v[15] = 0;
                    //println!("v[x]: {:?} v[y]: {:?}", self.v[x], self.v[y]);
                    let mut vy: usize;
                    let mut vx: usize;
                    for i in 0..n {
                        //if self.v[y] + n >= 32 {break;}
                        vy = ((self.v[y] + i) % 32) as usize;
                        let mut newByte = self.memory[(self.I + i as u16) as usize];
                        //println!("n {:?}", i);
                        //for 8 pixels
                        for p in 0..8 {
                            vx = ((self.v[x] + p) % 64) as usize;
                            //if self.v[x] + p >= 64 {break;}
                            //println!("p {:?}", p);
                            let mut bit: u8 = (newByte & 0b10000000) as u8;
                            bit = bit.rotate_left(1) as u8;
                            //println!("{:X}", bit);
                            newByte = newByte.rotate_left(1);
                            if bit == 1 {
                            //if bit == 1 && self.screen[(self.v[x] + p) as usize][(self.v[y] + n) as usize] == 1 {
                                if self.screen[vy][vx] == 1 {self.v[15] = 1;}
                                self.screen[vy][vx] ^= bit;
                            }

                            //self.screen[(self.v[x] + p) as usize][(self.v[y] + n) as usize] = bit;
                        }
                    }
                },
                add @ 0xE000 ... 0xEFFF => {
                    let x: u8 = (add & 0x0F00).rotate_right(8) as u8;
                    let nn : u16 = (add & 0x00FF) as u16;
                    match nn {
                        0x9E => {
                            if self.keys[self.v[x as usize] as usize] { self.pc += 2;}
                        },
                        0xA1 =>{
                            if !self.keys[self.v[x as usize] as usize] { self.pc += 2;}
                        },
                        _ => (),
                    }
                },
                add @ 0xF000 ... 0xFFFF => {
                    let x: usize = (add & 0x0F00).rotate_right(8) as usize;
                    let nn : u16 = (add & 0x00FF) as u16;
                    match nn {
                        0x07 => {self.v[x] = self.delay;},
                        0x0A => {
                            self.keyWait = true;
                            self.keyV = x;
                        },
                        0x15 => {self.delay = self.v[x];},
                        0x18 => {self.sound = self.v[x];},
                        0x1E => {self.I += self.v[x] as u16;},
                        0x29 => {self.I = (self.v[x] * 5) as u16;},
                        0x33 => {
                            let mut bcd = self.v[x];
                            self.memory[(self.I + 2) as usize] = bcd % 10;
                            bcd /= 10;
                            self.memory[(self.I + 1) as usize] = bcd % 10;
                            bcd /= 10;
                            self.memory[self.I as usize] = bcd % 10;
                        },
                        0x55 => {
                            for i in 0..(x+1) {
                                self.memory[self.I as usize] = self.v[i];
                                self.I += 1;
                            }
                        },
                        0x65 =>{
                            for i in 0..(x+1) {
                                self.v[i] = self.memory[self.I as usize];
                                self.I += 1;
                            }
                        },
                        _ => (),
                    }
                }
                _ => panic!("Unexpected opcode! {:X}", opcode),
            }
    }
}

fn main(){
    //initiate sdl2 stuff
    //sdl2::init(&[sdl2::InitFlag::Video, sdl2::InitFlag::Audio, sdl2::InitFlag::Timer]);
    let sdlContext = sdl2::init().unwrap();
    let video_subsystem = sdlContext.video().unwrap();
    //inits window
    let win = video_subsystem.window("Chip8", 64*scale, 32*scale)
        .position_centered().build().unwrap();
    let mut renderer = win.renderer().build().unwrap();

    //initiate chip8 vm
    let mut chip = CHIP8::new();
    let mut opCode : u16;

    let mut event_pump = sdlContext.event_pump().unwrap();
    chip.load("PONG".to_string());
    //for memAdd in 0..0xFF {
    //    println!("memAdd: {:X} memValue: {:X}", memAdd, chip.memory[(0x0200 + memAdd) as usize]);
    //}
    //opCode = chip.fetch();
    //println!("opcode fetched:{:X}", opCode);
    //opCode = chip.fetch();
    //println!("opcode fetched:{:X}", opCode);
    //opCode = chip.fetch();
    //println!("opcode fetched:{:X}", opCode);
    let mut slowestOP: u16 = 0;
    let mut slowestDuration: Duration = Duration::milliseconds(0);
    let mut startTime: PreciseTime;
    let mut endTime: PreciseTime;
    let mut dur: Duration;
    'main : loop {
        for event in event_pump.poll_iter(){
            match event {
                Event::Quit {..} => break 'main,
                //Event::None                  => break 'event,
                //eventually make work with keyboard mod
                Event::KeyDown {keycode: Some(keycode), ..} => chip.press(keycode, true),
                Event::KeyUp {keycode: Some(keycode), ..} => chip.press(keycode, false),
                _                            => {}
            }
        }

        opCode = chip.fetch();
        //println!("opcode fetched:{:X}", opCode);
        startTime = PreciseTime::now();
        chip.execute(opCode);
        endTime = PreciseTime::now();
        dur = startTime.to(endTime);
        if dur > slowestDuration {
            slowestOP = opCode;
            slowestDuration = dur;
        }
        println!("Slowest Opcode: {:X} Duration: {:?}", slowestOP, slowestDuration);
        //for display testing
        //chip.v[15] = 1;
        chip.display(&mut renderer);
    }
}
