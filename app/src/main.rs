mod utils;
mod invaders_bus;

use invaders_bus::InvadersBus;
use r8080::{cpu::{Interpreter8080, CPU8080}, Bus8080};
use utils::read_file_to_vec;

fn main() {
    // Read the ROM data.
    let mut invaders_bus = Box::new(InvadersBus::new());
    invaders_bus.write_buffer(0x0000, read_file_to_vec("roms/invaders.h"));
    invaders_bus.write_buffer(0x0800, read_file_to_vec("roms/invaders.g"));
    invaders_bus.write_buffer(0x1000, read_file_to_vec("roms/invaders.f"));
    invaders_bus.write_buffer(0x1800, read_file_to_vec("roms/invaders.e"));

    let mut cpu: Box<dyn CPU8080> = Box::new(Interpreter8080::new());
    cpu.set_bus(invaders_bus);
    cpu.run();
}
