use std::fs::File;
use std::io::Read;
use std::error::Error;
use chip8::Chip8;

mod ram;
mod chip8;
mod cpu;

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("data/INVADERS")?;
    let mut data = Vec::<u8>::new();
    f.read_to_end(&mut data)?;

    let mut chip8 = Chip8::new();
    chip8.load_rom(&mut data);

    loop {
        chip8.run_instruction();
    }
    Ok(())
}
