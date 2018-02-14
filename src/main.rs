extern crate sdl2;

mod cpu;

use std::env;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use cpu::{Chip8, HEIGHT, WIDTH};

const SCALE: u32 = 12;

fn main() {
    let rom = env::args().nth(1).expect("usage: chip8-rs <rom>");
    let mut chip = Chip8::new();
    chip.load_rom(&rom).expect("could not read rom");

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("chip8", WIDTH as u32 * SCALE, HEIGHT as u32 * SCALE)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut events = sdl.event_pump().unwrap();

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
                _ => {}
            }
        }

        chip.cycle();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (i, &on) in chip.gfx.iter().enumerate() {
            if on {
                let x = (i % WIDTH) as i32 * SCALE as i32;
                let y = (i / WIDTH) as i32 * SCALE as i32;
                canvas.fill_rect(Rect::new(x, y, SCALE, SCALE)).unwrap();
            }
        }
        canvas.present();
    }
}
