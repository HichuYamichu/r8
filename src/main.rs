use r8::cpu::CPU;
use sdl2;
use sdl2::event::Event;
use sdl2::pixels;
use sdl2::rect::Rect;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;

use r8::SCALE;
use r8::SCREEN_HEIGHT;
use r8::SCREEN_WIDTH;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("R8", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut f = File::open(filename).expect("no such file");
    let mut buffer = [0u8; 3584];
    let bytes_read = if let Ok(bytes_read) = f.read(&mut buffer) {
        bytes_read
    } else {
        0
    };

    assert!(bytes_read > 0);

    let mut cpu = CPU::new();
    cpu.load(&buffer);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => cpu.key_down(keycode),
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => cpu.key_up(keycode),
                _ => {}
            }
        }
        cpu.step();
        if cpu.redraw {
            for (y, row) in cpu.display.iter().enumerate() {
                for (x, &col) in row.iter().enumerate() {
                    let x = (x as u32) * SCALE;
                    let y = (y as u32) * SCALE;

                    canvas.set_draw_color(color(col));
                    let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, SCALE, SCALE));
                }
            }
            canvas.present();
        }
        thread::sleep(Duration::from_millis(2));
    }
}

fn color(value: u8) -> pixels::Color {
    if value == 0 {
        pixels::Color::RGB(0, 0, 0)
    } else {
        pixels::Color::RGB(0, 250, 0)
    }
}
