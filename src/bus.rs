use crate::keyboard::Keyboard;
//use crate::display::Display
use crate::ram::Ram;
use std::fmt;
use std::time;


pub struct Bus {
	ram: Ram,
	keyboard: Keyboard,
	//display: Display,
	delay_timer: u8,
	delay_timer_set_time: time::Instant
}

impl Bus {
	pub fn new() -> Bus {
		Bus {
			ram: Ram::new(),
			keyboard: Keyboard::new(),
			//display: Display::new(),
			delay_timer: 0,
			delay_timer_set_time: time::Instant::now()
		}
	}

	pub fn ram_read_byte(&self, address: u16) -> u8 {
		self.ram.read_byte(address)
	}

	pub fn ram_write_byte(&mut self, address: u16, value: u8) {
		self.ram.write_byte(address, value);
	}

	pub fn set_key_pressed(&mut self, key: Option<u8>) {
		self.keyboard.set_key_pressed(key);
	}

	pub fn get_key_pressed(&self) -> Option<u8> {
		self.keyboard.get_key_pressed()
	}

	pub fn is_key_pressed(&self, key_code: u8) -> bool {
		self.keyboard.is_key_pressed(key_code)
	}
}

