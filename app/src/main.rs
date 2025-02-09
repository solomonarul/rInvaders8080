// #![windows_subsystem = "windows"]

mod utils;
mod invaders_bus;

use sdl3::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};
use std::{cell::RefCell, rc::Rc, sync::{Arc, RwLock}, thread};

use invaders_bus::{InvadersBus, InvadersInputState};
use r8080::{cpu::{Interpreter8080, CPU8080}, Bus8080};
use utils::read_file_to_vec;

fn main() {
    // Init SDL.
    let context = sdl3::init().unwrap();
    let video_subsystem = context.video().unwrap();

    // Create a window.
    let window = video_subsystem.window("Space Invaders | Intel 8080", 220 * 3, 255 * 3)
        .position_centered().build().unwrap();

    let mut canvas = window.into_canvas();
    canvas.set_scale(3.0, 3.0).unwrap();

    // Read the ROM data into the bus and prepare the inputs.
    let input_state = Rc::new(RefCell::new(InvadersInputState{ first: 0x00, second: 0x00 }));
    let mut invaders_bus = Box::new(InvadersBus::new(input_state.clone())) as Box<dyn Bus8080>;
    invaders_bus.write_buffer(0x0000, read_file_to_vec("roms/invaders.h"));
    invaders_bus.write_buffer(0x0800, read_file_to_vec("roms/invaders.g"));
    invaders_bus.write_buffer(0x1000, read_file_to_vec("roms/invaders.f"));
    invaders_bus.write_buffer(0x1800, read_file_to_vec("roms/invaders.e"));

    // Create the CPU and attach the Bus.
    let shared_bus = Arc::new(RwLock::new(invaders_bus));
    let mut cpu = Box::new(Interpreter8080::new()) as Box<dyn CPU8080>;
    cpu.set_bus(Arc::clone(&shared_bus));

    // Create a thread for the CPU to run on.
    let shared_cpu = Arc::new(RwLock::new(cpu));
    let cpu_thread = thread::spawn({
        let shared_bus = Arc::clone(&shared_bus);
        let shared_cpu = Arc::clone(&shared_cpu);
        let spin_sleeper = spin_sleep::SpinSleeper::default();
        move || {
            println!("[INFO]: Emulation thread started.");
            'main: loop {
                let mut count = 0f64;
                while count < 1f64 / 120.0 {
                    let mut cpu = shared_cpu.write().unwrap();
                    let last_cycles = cpu.get_executed_cycles();
                    cpu.step();
                    let current_cycles = cpu.get_executed_cycles();
                    if (current_cycles / 16666) % 2 == 0 && (current_cycles / 16666) % 2 != (last_cycles / 16666) % 2 {
                        shared_bus.write().unwrap().push_interrupt(0xCF);
                    }
                    if (current_cycles / 16666) % 2 == 1 && (current_cycles / 16666) % 2 != (last_cycles / 16666) % 2 {
                        shared_bus.write().unwrap().push_interrupt(0xD7);          
                    }
                    if !cpu.is_running() { break 'main; }
                    count += ((current_cycles - last_cycles) as f64) / 2000000.0
                }
                spin_sleeper.sleep_s(1f64 / 120.0);
            }
            println!("[INFO]: Emulation thread stopped.");
        }
    });

    // App's main loop.
    let spin_sleeper = spin_sleep::SpinSleeper::default();
    let mut event_pump = context.event_pump().unwrap();
    'main: loop {
        // Forcefully quit the app if somehow our emulator finishes running.
        if cpu_thread.is_finished() {
            break 'main
        }

        // Clear the background.
        canvas.set_draw_color(Color::RGB(10, 10, 10));
        canvas.clear();

        // Draw the output.
        {
            let mut point = Point::new(0, 0);
            let bus = shared_bus.read().unwrap();
            while point.x < 220 {
                point.y = 0;
                while point.y < 255 {
                    match point.y {
                        10..=34 => { canvas.set_draw_color(Color::RGB(245, 245, 150)); }
                        35..=50 => { canvas.set_draw_color(Color::RGB(245, 100, 100)); }
                        193..=239 => { canvas.set_draw_color(Color::RGB(100, 245, 100)); }
                        241..=255 => { canvas.set_draw_color(Color::RGB(100, 245, 100)); }
                        _ => { canvas.set_draw_color(Color::RGB(225, 225, 245)); }
                    }

                    let position = point.x * 256 + (256 - point.y);
                    let pixel = bus.read_b((0x2400 + position / 8) as u16) & (1 << (position % 8));
                    if pixel != 0 {
                        canvas.draw_point(point).unwrap();
                    }
                    point.y += 1;
                }
                point.x += 1;
            }
            canvas.present();
        }

        // Poll events.
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                }
                // Coin button.
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first |= 0b00000001;
                }
                // 1-P buttons.
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first |= 0b00010000;
                }
                Event::KeyUp { keycode: Some(Keycode::W), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first &= 0b11101111;
                }
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first |= 0b00100000;
                }
                Event::KeyUp { keycode: Some(Keycode::A), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first &= 0b11011111;
                }    
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first |= 0b01000000;
                }
                Event::KeyUp { keycode: Some(Keycode::D), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first &= 0b10111111;
                }  
                // 1-P button.
                Event::KeyDown { keycode: Some(Keycode::_1), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first |= 0b00000100;
                }
                Event::KeyUp { keycode: Some(Keycode::_1), .. } => {
                    let mut state = input_state.borrow_mut();
                    state.first &= 0b11111011;
                }
                _ => {}
            }
        }

        // Render less.
        spin_sleeper.sleep_s(1f64 / 60.0);
    }

    // Stop the CPU.
    {
        let mut cpu = shared_cpu.write().unwrap();
        cpu.stop();
    }
    cpu_thread.join().unwrap();
    println!("[INFO]: Main thread stopped.");
}
