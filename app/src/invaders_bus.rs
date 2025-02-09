use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use r8080::{cpu::Registers, Bus8080};

pub struct InvadersInputState {
    pub first: u8,
    pub second: u8
}

pub struct InvadersBus
{
    rom: [u8; 0x2000],
    ram: [u8; 0x400],
    vram: [u8; 0x1C00],
    shifter: u16,
    offset: u8,
    inputs: Rc<RefCell<InvadersInputState>>,
    interrupts: VecDeque<u8>
}

impl InvadersBus
{
    pub fn new(inputs: Rc<RefCell<InvadersInputState>>) -> Self {
        Self {
            rom: [0x00; 0x2000],
            ram: [0x00; 0x400],
            vram: [0x00; 0x1C00],
            shifter: 0x0000,
            offset: 0x00,
            interrupts: VecDeque::new(),
            inputs
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
unsafe impl Send for InvadersBus {}

impl Bus8080 for InvadersBus
{
    fn get_interrupt(&mut self) -> u8 {
        self.interrupts.pop_front().unwrap()
    }

    fn has_interrupt(&self) -> bool {    
        self.interrupts.front() != None
    }

    fn push_interrupt(&mut self, b: u8) {
        self.interrupts.push_back(b);
    }

    // Refference: https://computerarcheology.com/Arcade/SpaceInvaders/Hardware.html
    fn in_b(&mut self, _: &mut Registers, b: u8) -> u8 {
        match b {
            0x1 => {
                let value = self.inputs.borrow().first;
                self.inputs.borrow_mut().first &= 0xFE;
                0b10001000 | value
            }
            0x2 => {
                0b00001000 | self.inputs.borrow().second
            }
            0x3 => {
                // TODO: WHY 10??? Display draws wrong otherwise
                ((self.shifter >> (10 - self.offset)) & 0xFF) as u8
            }
            0x6 => { /* Watchdog does nothing for us. */ 0xFF }
            _ => {
                // println!("[INFO]: Unhandled read, returned 0xFF from device {:02X} on InvadersBus.", b);
                0xFF
            }
        }
    }

    fn out_b(&mut self, _: &mut Registers, b: u8, a: u8) {
        match b {
            0x2 => {
                self.offset = b & 0b111;
            }
            0x4 => {
                self.shifter >>= 8;
                self.shifter |= (a as u16) << 8;
            }
            0x6 => { /* Watchdog does nothing for us. */ }
            _ => {
                // println!("[INFO]: Unhandled write {:02X} to device {:02X} on InvadersBus.", a, b);
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