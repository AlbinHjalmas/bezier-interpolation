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

use flo_curves::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

use colored::*;

extern crate nalgebra as na;
use na::{Const, Dynamic, ArrayStorage, VecStorage, Matrix, DMatrix, DimAdd};

type MatDyn = Matrix<f32, Dynamic, Dynamic, VecStorage<f32, Dynamic, Dynamic>>;
type VecDyn = Matrix<f32, Const<1>, Dynamic, VecStorage<f32, Const<1>, Dynamic>>;
type Mat2D = Matrix<f32, Dynamic, Const<2>, VecStorage<f32, Dynamic, Const<2>>>;
type Vec2D = Matrix<f32, Const<1>, Const<2>, ArrayStorage<f32, 1, 2>>;


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

/// A cubic bezier curve has control points P0, P1, P2 and P3. 
/// For a bezier curve to run through the points input to this function
/// we need to find the control points 
fn get_bezier_coef(points: &Vec<Vec2D>) -> (Vec<Vec2D>, Vec<Vec2D>) {
    // since the formulas work given that we have n+1 points
    // then n must be this:
    let n = points.len() - 1;

    // Build coefficients matrix
    let mut coeffs: MatDyn = MatDyn::identity(n, n).scale(4.0);
    coeffs.slice_mut((1, 0), (n - 1, n)).fill_diagonal(1.0);
    coeffs.slice_mut((0, 1), (n, n - 1)).fill_diagonal(1.0);
    coeffs[(0, 0)] = 2.0;
    coeffs[(n - 1, n - 1)] = 7.0;
    coeffs[(n - 1, n - 2)] = 2.0;

    // Build points matrix
    let mut points_vector: Vec<Vec2D> = points.windows(2).map(|vectors| {
        2.0 * (2.0 * vectors[0] + vectors[1])
    }).collect();
    points_vector[0] = points[0] + 2.0 * points[1];
    points_vector[n - 1] = 8.0 * points[n - 1] + points[n];
    let points_matrix = Mat2D::from_rows(points_vector.as_slice());

    // Solve system for p1 points
    let qr_decomp = coeffs.qr();
    let p1 = qr_decomp.solve(&points_matrix).expect("Failed to solve system...");
    let p1_vec: Vec<Vec2D> = p1.row_iter().map(|row| Vec2D::from_rows(&[row])).collect();

    // Calculate p2 points
    let mut p2_vec: Vec<Vec2D> = Vec::new();
    for i in 0..(n - 1) {
        p2_vec.push(2.0 * points[i + 1] - p1_vec[i + 1]);
    }
    p2_vec.push((p1_vec[n - 1] + points[n]) / 2.0);

    return (p1_vec, p2_vec);
}

fn get_bezier_cubic(p0: Vec2D, p1: Vec2D, p2: Vec2D, p3: Vec2D) -> Box<dyn Fn(f32) -> Vec2D> {
    Box::new(
        move |t| {
            let one_minus_t = 1.0 - t;
            one_minus_t.powi(3) * p0 + 3.0 * t * one_minus_t.powi(2) * p1 + 3.0 * t.powi(2) * one_minus_t * p2 + t.powi(3) * p3
        }
    )
}

fn get_bezier_segments(points: &Vec<Vec2D>) -> Vec<Box<dyn Fn(f32) -> Vec2D>> {
    let (p1, p2) = get_bezier_coef(points);
    let mut i = 0usize;
    points.windows(2).map(|p| {
        let ret = get_bezier_cubic(p[0], p1[i], p2[i], p[1]);
        i += 1;
        ret
    }).collect()
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

    // Add points for Bezier test
    /*
    window_state.circle_pos.push(Vec2D::new(50.0, 300.0));        // P1
    window_state.circle_pos.push(Vec2D::new(400.0, 100.0));       // P2
    window_state.circle_pos.push(Vec2D::new(700.0, 500.0));       // P3
    window_state.circle_pos.push(Vec2D::new(1150.0, 300.0));      // P4
    */

    window.run_loop(window_state);
}