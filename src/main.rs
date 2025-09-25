use chip8::{Chip8, DISP_HEIGHT, DISP_WIDTH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::env;

mod chip8;

extern crate sdl2;
const SCALE: u32 = 15;
const TICKS_PER_FRAME: u32 = 10;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: Missing parameter - path/to/game");
        return;
    }
    let mut chip8 = Chip8::new();
    chip8.load_rom(&args[1]);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Chip-8",
            DISP_WIDTH as u32 * SCALE,
            DISP_HEIGHT as u32 * SCALE,
        )
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = match_key(key) {
                        chip8.set_key_value(k, 1)
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = match_key(key) {
                        chip8.set_key_value(k, 0)
                    }
                }
                _ => {}
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.cycle();
        }
        chip8.tick_timers();

        draw(&chip8, &mut canvas);
    }

    println!("Finito.")
}

fn draw(chip8: &Chip8, canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for x in 0..DISP_WIDTH {
        for y in 0..DISP_HEIGHT {
            if chip8.video[y * DISP_WIDTH + x] {
                canvas
                    .fill_rect(Rect::new(
                        (x * SCALE as usize) as i32,
                        (y * SCALE as usize) as i32,
                        SCALE,
                        SCALE,
                    ))
                    .expect("Error when drawing");
            }
        }
    }
    canvas.present();
}

fn match_key(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
