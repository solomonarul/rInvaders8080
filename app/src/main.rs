mod utils;
mod invaders_bus;

use sdl3::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};
use std::{sync::{Arc, RwLock}, thread, time::Duration};

use invaders_bus::InvadersBus;
use r8080::{cpu::{Interpreter8080, CPU8080}, Bus8080};
use utils::read_file_to_vec;

fn main() {
    // Init SDL.
    let context = sdl3::init().unwrap();
    let video_subsystem = context.video().unwrap();

    // Create a window.
    let window = video_subsystem.window("Space Invaders | Intel 8080", 224 * 3, 256 * 3)
        .position_centered().build().unwrap();

    let mut canvas = window.into_canvas();
    canvas.set_scale(3.0, 3.0).unwrap();
    
    // Read the ROM data into the bus..
    let mut invaders_bus = Box::new(InvadersBus::new()) as Box<dyn Bus8080>;
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
        let shared_cpu = Arc::clone(&shared_cpu);
        move || {
            loop {
                let mut cpu = shared_cpu.write().unwrap();
                cpu.step();

                if !cpu.is_running() { break; }
            }
        }
    });

    // Our app's main loop.
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
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            let bus = shared_bus.read().unwrap();
            for x in 0..224 {
                for y in 0..256 {
                    let pixel = bus.read_b(0x2400 + (x * 256 + (256 - y)) / 8) & (1 << ((x * 256 + (256 - y)) % 8));
                    if pixel != 0 {
                        canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
                    }
                }
            }
            canvas.present();
        }

        // Poll events.
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                },
                _ => {}
            }
        }

        // Render less.
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 62));
    }

    // Stop the CPU.
    {
        let mut cpu = shared_cpu.write().unwrap();
        cpu.stop();
    }
    cpu_thread.join().unwrap();
}
