use crate::bus::Bus;
use std::fmt;
use rand::Rng;
use rand::rngs::ThreadRng;

pub const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    vx: [u8; 16],
    pc: u16,
    i: u16,
    ret_stack: Vec<u16>,
    rng: ThreadRng,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            vx: [0; 16],
            pc: PROGRAM_START,
            i: 0,
            ret_stack: Vec::<u16>::new(),
            rng: rand::thread_rng(),
        }
    }

    pub fn run_instruction(&mut self, bus: &mut Bus) {
        let hi = bus.ram_read_byte(self.pc) as u16;
        let lo = bus.ram_read_byte(self.pc+1) as u16;
        let instruction: u16 = (hi << 8) | lo;
        println!("Instruction read {:#X}: hi:{:#X} lo:{:#X}", instruction, hi, lo);

        let nnn = instruction & 0x0fff;
        let nn = (instruction & 0x00ff) as u8;
        let n = (instruction & 0x000f) as u8;
        let x = (nnn >> 8) as u8;
        let y = nn >> 4;

        match instruction >> 12 {
            0x0 => {
                match nn {
                    0xe0 => {
                        // clear display screen
                        bus.clear_screen();
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
                let rand_number = self.rng.gen_range(0, 255);
                self.write_reg_vx(x, rand_number & nn);
                self.pc += 2;
            },
            0xd => {
                // draw sprite
                let vx = self.read_reg_vx(x);
                let vy = self.read_reg_vx(y);
                self.debug_draw_sprite(bus, vx, vy, n);
                self.pc += 2;
            },
            0xe => {
                match nn {
                    0x9e => {
                        let key_vx = self.read_reg_vx(x);
                        if bus.is_key_pressed(key_vx) {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    0xa1 => {
                        let key_vx = self.read_reg_vx(x);
                        if !bus.is_key_pressed(key_vx) {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    _ => panic!(
                        "Unrecognized 0xe*** instruction {:#X}:{:#X}",
                        self.pc,
                        instruction
                    ),
                }
            },
            0xf => {
                match nn {
                    0x07 => {
                        self.write_reg_vx(x, bus.get_delay_timer());
                        self.pc += 2;
                    },
                    0x0a => {
                        if let Some(val) = bus.get_key_pressed() {
                            self.write_reg_vx(x, val);
                            self.pc += 2;
                        }
                    },
                    0x15 => {
                        bus.set_delay_timer(self.read_reg_vx(x));
                        self.pc += 2;
                    },
                    0x18 => {
                        // no sound timer yet
                        self.pc += 2;
                    },
                    0x1e => {
                        self.i += (self.read_reg_vx(x) as u16);
                        self.pc += 2;
                    },
                    0x29 => {
                        //i == sprite address for character in Vx
                        //Multiply by 5 because each sprite has 5 lines, each line
                        //is 1 byte.
                        self.i = self.read_reg_vx(x) as u16 * 5;
                        self.pc += 2;
                    },
                    0x33 => {
                        let vx = self.read_reg_vx(x);
                        bus.ram_write_byte(self.i, vx / 100);
                        bus.ram_write_byte(self.i + 1, (vx % 100) / 10);
                        bus.ram_write_byte(self.i + 2, vx % 10);
                        self.pc += 2;
                    },
                    0x55 => {
                        for index in 0..x + 1 {
                            let value = self.read_reg_vx(index);
                            bus.ram_write_byte(self.i + index as u16, value);
                        }
                        self.i += x as u16 + 1;
                        self.pc += 2;
                    },
                    0x65 => {
                         for index in 0..x + 1 {
                            let value = bus.ram_read_byte(self.i + index as u16);
                            self.write_reg_vx(index, value);
                        }
                        self.i += x as u16 + 1;
                        self.pc += 2;
                    },
                    _ => panic!(
                        "Unrecognized 0xf*** instruction {:#X}:{:#X}",
                        self.pc,
                        instruction
                    ),
                }
            },
            _ => panic!(
                "Unrecognized instruction {:#X}:{:#X}",
                self.pc,
                instruction
            ),
        }
    }

    pub fn write_reg_vx(&mut self, index: u8, value:u8) {
        self.vx[index as usize] = value;
    }
 
    pub fn read_reg_vx(&mut self, index: u8) -> u8 {
        self.vx[index as usize]
    }

    fn debug_draw_sprite(&mut self, bus: &mut Bus, x: u8, y: u8, height: u8) {
        let mut should_set_vf = false;
        for sprite_y in 0..height {
            let b = bus.ram_read_byte(self.i + sprite_y as u16);
            if bus.debug_draw_byte(b, x, y + sprite_y) {
                should_set_vf = true;
            }
        }
        if should_set_vf {
            self.write_reg_vx(0xF, 1);
        } else {
            self.write_reg_vx(0xF, 0);
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\npc: {:#X}\n", self.pc)?;
        write!(f, "vx: ")?;
        for item in &self.vx {
            write!(f, "{:#X}", *item)?;
        }
        write!(f, "\n")?;
        write!(f, "i: {:#X}\n", self.i)
    }
}
