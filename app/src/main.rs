#![windows_subsystem = "windows"]

mod utils;
mod invaders_bus;

use sdl3::{event::Event, keyboard::Keycode, messagebox::MessageBoxFlag, pixels::Color, rect::Point};
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
    let rom_data = [(0x0000, "roms/invaders.h"), (0x0800, "roms/invaders.g"), (0x1000, "roms/invaders.f"), (0x1800, "roms/invaders.e")];
    for rom in rom_data {
        let read_result = read_file_to_vec(rom.1);
        if read_result.is_err() {
            println!("[EROR]: Couldn't find ROM at path: {}", rom.1);
            sdl3::messagebox::show_simple_message_box(MessageBoxFlag::ERROR, "Error!", format!("Couldn't find ROM at path: {}", rom.1).as_str(), None).unwrap();
            return;
        }
        invaders_bus.write_buffer(rom.0, read_result.unwrap());
    }

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
                let cpu_freq = 2000000;
                let cycles_per_vblank = (cpu_freq / 60) / 2;
                let mut timer = 0f64;

                // Just a bit faster than the screen.
                let refresh_rate = 1f64 / 90.0;
                while timer < refresh_rate {
                    // CPU catching up.
                    let mut cpu = shared_cpu.write().unwrap();
                    let last_cycles = cpu.get_executed_cycles();
                    cpu.step();
                    let current_cycles = cpu.get_executed_cycles();
                    timer += ((current_cycles - last_cycles) as f64) / cpu_freq as f64;
                    
                    // Interrupt handling per half vblank.
                    let current_vblanks = current_cycles / cycles_per_vblank;
                    let last_vblanks = last_cycles / cycles_per_vblank;
                    if current_vblanks % 2 == 0 && current_vblanks % 2 != last_vblanks % 2 {
                        shared_bus.write().unwrap().push_interrupt(0xCF);
                    }
                    if current_vblanks % 2 == 1 && current_vblanks % 2 != last_vblanks % 2 {
                        shared_bus.write().unwrap().push_interrupt(0xD7);        
                    }
                    
                    // If we are not running, stop this thread.
                    if !cpu.is_running() { break 'main; }
                }
                spin_sleeper.sleep_s(refresh_rate);
            }
            println!("[INFO]: Emulation thread stopped.");
        }
    });

    // App's main loop.
    let mut event_pump = context.event_pump().unwrap();
    let spin_sleeper = spin_sleep::SpinSleeper::default();
    'main: loop {
        // Forcefully quit the app if somehow our emulator finishes running before this thread.
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
                point.y = 0;    // Keep in mind that when displayed, the screen is rotated.
                // Scanline matching.
                while point.y < 255 {
                    // Match the colors per area.
                    match point.y {
                        10..=34 => { canvas.set_draw_color(Color::RGB(210, 210, 230)); }                            // Upper light gray.
                        35..=50 => { canvas.set_draw_color(Color::RGB(245, 100, 100)); }                            // Upper red.
                        193..=224 => { canvas.set_draw_color(Color::RGB(100, 200, 100)); }                          // Lower-mid green.
                        225..=239 => { canvas.set_draw_color(Color::RGB(245, 100, 100)); }                          // Lower red.
                        241..=255 => {
                            if point.x > 15 && point.x < 110 { canvas.set_draw_color(Color::RGB(100, 200, 100)); }  // Lower green.
                            else { canvas.set_draw_color(Color::RGB(225, 225, 255)); }
                        }
                        _ => { canvas.set_draw_color(Color::RGB(225, 225, 255)); }                                  // Normal white.
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

        // Render thread matching.
        spin_sleeper.sleep_s(1f64 / 60.0);
    }

    // Stop the CPU thread, app is done.
    {
        let mut cpu = shared_cpu.write().unwrap();
        cpu.stop();
    }
    cpu_thread.join().unwrap();
    println!("[INFO]: Main thread stopped.");
}
