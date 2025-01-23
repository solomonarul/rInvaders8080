use r8080::{cpu::Registers, Bus8080};

pub struct InvadersBus
{
    rom: [u8; 0x2000],
    ram: [u8; 0x400],
    vram: [u8; 0x1C00]
}

#[allow(dead_code)]
impl InvadersBus
{
    pub fn new() -> Self {
        Self {
            rom: [0x00; 0x2000],
            ram: [0x00; 0x400],
            vram: [0x00; 0x1C00]
        }
    }

    pub fn dump_range(&self, start: u16, end: u16) {
        for (index, value) in self.ram[start as usize..end as usize].iter().enumerate() {
            println!("{:04X}: {:02X}", index + start as usize, value)
        }
    }

    fn write_b_unrestricted(&mut self, a: u16, b: u8) {
        match a {
            0x0000..=0x1FFF => {
                self.rom[a as usize] = b
            }
            0x2000..=0x23FF => {
                self.ram[(a - 0x2000) as usize] = b;
            }
            0x2400..=0x3FFF => {
                self.vram[(a - 0x2400) as usize] = b;
            }
            0x4000.. => {
                self.ram[((a - 0x4000) % 0x400) as usize] = b;
            }
        }
    }
}

unsafe impl Sync for InvadersBus {}

impl Bus8080 for InvadersBus
{
    fn in_b(&mut self, _: &mut Registers, b: u8) -> u8 {
        println!("[INFO]: Unhandled read, returned 0xFF from device {:02X} on InvadersBus.", b);
        0xFF
    }

    fn out_b(&mut self, _: &mut Registers, b: u8, a: u8) {
        match b {
            0x6 => { /* TODO: implement watchdog properly, this should suffice for now. */ }
            _ => {
                println!("[INFO]: Unhandled write {:02X} to device {:02X} on InvadersBus.", a, b);
            }
        }
    }

    fn read_b(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x1FFF => {
                self.rom[a as usize]
            }
            0x2000..=0x23FF => {
                self.ram[(a - 0x2000) as usize]
            }
            0x2400..=0x3FFF => {
                self.vram[(a - 0x2400) as usize]
            }
            0x4000.. => {
                self.ram[((a - 0x4000) % 0x400) as usize]
            }
        }
    }

    fn read_w(&self, a: u16) -> u16 {
        return ((self.read_b(a + 1) as u16) << 8)  | self.read_b(a) as u16;
    }

    fn write_b(&mut self, a: u16, b: u8) {
        match a {
            0x2000..=0x23FF => {
                self.ram[(a - 0x2000) as usize] = b;
            }
            0x2400..=0x3FFF => {
                self.vram[(a - 0x2400) as usize] = b;
            }
            0x4000.. => {
                self.ram[((a - 0x4000) % 0x400) as usize] = b;
            }
            _ => {}
        }
    }

    fn write_w(&mut self, a: u16, w: u16) {
        self.write_b(a + 1, (w >> 8) as u8);
        self.write_b(a, (w & 0xFF) as u8);
    }

    fn write_buffer(&mut self, a: u16, data: Vec<u8>) {
        for (index, value) in data.iter().enumerate() {
            self.write_b_unrestricted(u16::try_from(a as usize + index).unwrap(), *value);
        }
    }
}