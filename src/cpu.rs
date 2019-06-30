use ram::Ram;
use std::fmt;
use rand;
use rand::distributions::{IndependentSample, Range};

pub const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    vx: [u8; 16],
    pc: u16,
    i: u16,
    ret_stack: Vec<u16>,
    rng: rand::ThreadRng,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            vx: [0; 16],
            pc: PROGRAM_START,
            i: 0,
        }
    }

    pub fn run_instruction(&mut self, ram: &mut Ram) {
        let hi = ram.read_byte(self.pc) as u16;
        let lo = ram.read_byte(self.pc+1) as u16;
        let instruction: u16 = (hi << 8) | lo;
        println!("Instruction read {:#X}: hi:{:#X} lo:{:#X}", instruction, hi, lo);

        let nnn = instruction & 0x0fff;
        let nn = (instruction & 0x00ff) as u8;
        let n = instruction & 0x000f as u8;
        let x = (nnn >> 8) as u8;
        let y = nn >> 4;

        match instruction >> 12 {
            0x0 => {
                match nn {
                    0xe0 => {
                        // clear display screen
                        self.pc += 2;
                    },
                    0xee => {
                        // return from subroutine
                        self.pc = self.ret_stack.pop().unwrap();
                    },
                    _ => panic!(
                        "Unrecognized 0x00** instruction {:#X}:{:#X}",
                        self.pc,
                        instruction
                    ),
                }
            },
            0x1 => {
                // jump to nnn
                self.pc = nnn;
            },
            0x2 => {
                // call subroutine at nnn
                self.ret_stack.push(self.pc + 2);
                self.pc = nnn;
            },
            0x3 => {
                // skip next instruction
                let vx = self.read_reg_vx(x);
                if vx == nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0x4 => {
                // skip next instruction
                let vx = self.read_reg_vx(x);
                if vx != nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0x5 => {
                match n {
                    0x0 => {
                        // skip next instruction
                        let vx = self.read_reg_vx(x);
                        let vy = self.read_reg_vx(y);
                        if vx == vy {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    _ => panic!(
                        "Unrecognized 0x5*** instruction {:#X}:{:#X}",
                        self.pc,
                        instruction
                    ),
                }
            },
            0x6 => {
                // vx = nn
                self.write_reg_vx(x, nn);
                self.pc += 2;
            },
            0x7 => {
                let mut vx = self.read_reg_vx(x);
                self.write_reg_vx(x, vx.wrapping_add(nn));
                self.pc += 2;
            },
            0x8 => {
                let vx = self.read_reg_vx(x);
                let vy = self.read_reg_vx(y);

                match n {
                    0x0 => self.write_reg_vx(x, vy),
                    0x1 => self.write_reg_vx(x, vx | vy),
                    0x2 => self.write_reg_vx(x, vx & vy),
                    0x3 => self.write_reg_vx(x, vx ^ vy),
                    0x4 => {
                        let sum = vx as u16 + vy as u16;
                        self.write_reg_vx(x, sum as u8);
                        let flag: u8 = if sum > 0xff {
                            1
                        } else {
                            0
                        };
                        self.write_reg_vx(0xf, flag);
                    },
                    0x5 => {
                        let diff: i8 = vx as i8 - vy as i8;
                        self.write_reg_vx(x, diff as u8);
                        let flag: u8 = if diff < 0 {
                            0
                        } else {
                            1
                        };
                        self.write_reg_vx(0xf, flag);
                    },
                    0x6 => {
                        self.write_reg_vx(0xf, vx & 0x01);
                        self.write_reg_vx(x, vx >> 1);
                    },
                    0x7 => {
                        let diff: i8 = vy as i8 - vx as i8;
                        self.write_reg_vx(x, diff as u8);
                        let flag: u8 = if diff < 0 {
                            0
                        } else {
                            1
                        };
                        self.write_reg_vx(0xf, flag);
                    },
                    0xe => {
                        self.write_reg_vx(0xf, vx >> 7);
                        self.write_reg_vx(x, vx << 1);
                    },
                    _ => panic!(
                        "Unrecognized 0x8XY* instruction {:#X}:{:#X}",
                        self.pc,
                        instruction
                    ),
                }

                self.pc += 2;
            },
            0x9 => {
                match n {
                    0x0 => {
                        // skip next instruction
                        let vx = self.read_reg_vx(x);
                        let vy = self.read_reg_vx(y);
                        if vx != vy {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    _ => panic!(
                        "Unrecognized 0x9*** instruction {:#X}:{:#X}",
                        self.pc,
                        instruction
                    ),
                }
            },
            0xa => {
                self.i = nnn;
                self.pc += 2;
            },
            0xb => {
                let v0 = self.read_reg_vx(0) as u16;
                self.pc += (v0 + nnn);
            },
            0xc => {
                let interval = Range::new(0, 255);
                let number = interval.ind_sample(&mut self.rng);
                self.write_reg_vx(x, number & nn);
                self.pc += 2;
            },
            0xd => {
                // draw sprite
                let vx = read_reg_vx(x);
                let vy = read_reg_vx(y);
                self.pc += 2;
            },
            0xe => {},
            0xf => {},
        }
    }

    pub fn write_reg_vx(&mut self, index: u8, value:u8) {
        self.vx[index as usize] = value;
    }
 
    pub fn.read_reg_vx(&mut self, index: u8) {
        self.vx[index as usize]
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\npc: {:#X}\n", self.pc)?;
        write!(f, "vx: ")?
        for item in &self.vx {
            write!(f, "{:#X}", *item)?;
        }
        write!(f, "\n")?;
        write!(f, "i: {:#X}\n", self.i)
    }
}
