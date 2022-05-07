mod bezier_interpolation;
use bezier_interpolation::*;

use std::convert::identity;
use std::fmt::Pointer;
use std::ops::Mul;
use std::rc::Rc; 
use std::time::{Instant, Duration};
use std::string::String;

use num::traits::real;
use speedy2d::Window;
use speedy2d::window::{WindowHelper, WindowHandler, MouseButton};
use speedy2d::color::Color;
use speedy2d::Graphics2D;
use speedy2d::font::{TextLayout, TextOptions, Font, FormattedTextBlock};
use speedy2d::dimen::Vector2;

struct PointerStatus {
    position: Vector2<f32>,
    l_btn_pushed: bool,
    r_btn_pushed: bool
}

impl PointerStatus {
    fn new() -> PointerStatus {
        PointerStatus {position: Vector2::new(0.0, 0.0), l_btn_pushed: false, r_btn_pushed: false}
    }
}

struct AbbesWindowHandler {
    font: Font,
    str_buffer: String,
    pointer_status: PointerStatus,
    circle_pos: Vec<Vec2D>,
    prev_time: Instant,
    accumulated_duration: Duration,
    accumulated_interpolation_duration: Duration,
    iterations: usize
}

impl WindowHandler for AbbesWindowHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D)
    {
        graphics.clear_screen(Color::WHITE);

        let curr_time = std::time::Instant::now();
        let duration = curr_time.duration_since(self.prev_time);
        
        if self.iterations % 100 == 0 {
            let avg_frame_rate = self.iterations as f64 / self.accumulated_duration.as_secs_f64();
            println!("Framerate: {}", avg_frame_rate);
        }

        for point in &self.circle_pos {
            graphics.draw_circle(Vector2::new(point[0], point[1]), 7.5, Color::RED);
        }

        let interpolation_start = std::time::Instant::now();
        if self.circle_pos.len() > 2 {
            const WIDTH: f32 = 6.0;
            const STEP: f32 = 1.0 / 50 as f32;
            let segments = get_bezier_segments(&self.circle_pos);
            for segment in segments {
                let mut prev_point = segment(0.0);
                graphics.draw_circle((prev_point[0], prev_point[1]), WIDTH/2.0, Color::BLUE);
                let mut t = STEP;
                while t <= 1.0 + STEP / 2.0 {
                    let point = segment(t);
                    graphics.draw_line((prev_point[0], prev_point[1]), (point[0], point[1]), WIDTH, Color::BLUE);
                    graphics.draw_circle((point[0], point[1]), WIDTH/2.0, Color::BLUE);
                    t += STEP;
                    prev_point = point;
                }
            }
        }
        let interpolation_end = std::time::Instant::now();
        self.accumulated_interpolation_duration += interpolation_end.duration_since(interpolation_start);
        if self.iterations % 100 == 0 {
            println!("Average interpolation time: {}", self.accumulated_interpolation_duration.as_secs_f64() / 100 as f64);
            self.accumulated_interpolation_duration = Duration::new(0, 0);
        }


        // Store the time to be able to measure duration
        self.iterations += 1;
        self.prev_time = curr_time;
        self.accumulated_duration += duration;
        // Request that we draw another frame once this one has finished
        helper.request_redraw();
    }

    fn on_mouse_move(&mut self, helper: &mut WindowHelper<()>, position: Vector2<f32>) {
        self.pointer_status.position = position;

        if self.pointer_status.l_btn_pushed {
            // self.circle_pos.push(position);
        }
    }

    fn on_mouse_button_down(&mut self, helper: &mut WindowHelper<()>, button: MouseButton) {
        // println!("{} on_mouse_button_down", "Callback: ".bold().green());
        // println!("{:?}", button);

        match button {
            MouseButton::Left => {
                self.pointer_status.l_btn_pushed = true;
                self.circle_pos.push(Vec2D::new(self.pointer_status.position.x, self.pointer_status.position.y));
            },
            MouseButton::Right => {
                self.pointer_status.r_btn_pushed = true;
                self.circle_pos.pop();
            },
            _ => println!(" ")
        }
    }

    fn on_mouse_button_up(&mut self, helper: &mut WindowHelper<()>, button: speedy2d::window::MouseButton) {
        // println!("{} on_mouse_button_up", "Callback: ".bold().green());
        // println!("{:?}", button);

        match button {
            MouseButton::Left => self.pointer_status.l_btn_pushed = false,
            MouseButton::Right => self.pointer_status.r_btn_pushed = false,
            _ => return
        }
    }

    fn on_keyboard_char(&mut self, helper: &mut WindowHelper<()>, unicode_codepoint: char) {
        if unicode_codepoint == '\u{8}' {
            self.str_buffer.pop();
        } else {
            self.str_buffer.push(unicode_codepoint);
        }
    }
}

fn main() {
    let window = Window::new_centered("Abbes testfönster <3", (1200, 600)).unwrap();
    let font = Font::new(include_bytes!("../assets/Roboto-Regular.ttf")).unwrap();
    let mut window_state = AbbesWindowHandler {
        font,
        str_buffer: String::new(),
        pointer_status: PointerStatus::new(), 
        circle_pos: Vec::new(), 
        prev_time: Instant::now(),
        accumulated_duration: Duration::new(0, 0),
        accumulated_interpolation_duration: Duration::new(0, 0),
        iterations: 0
    };

    window.run_loop(window_state);
}