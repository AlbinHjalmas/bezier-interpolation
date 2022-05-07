extern crate nalgebra as na;
use na::{Const, Dynamic, ArrayStorage, VecStorage, Matrix, DMatrix, DimAdd};

pub type MatDyn = Matrix<f32, Dynamic, Dynamic, VecStorage<f32, Dynamic, Dynamic>>;
pub type VecDyn = Matrix<f32, Const<1>, Dynamic, VecStorage<f32, Const<1>, Dynamic>>;
pub type Mat2D = Matrix<f32, Dynamic, Const<2>, VecStorage<f32, Dynamic, Const<2>>>;
pub type Vec2D = Matrix<f32, Const<1>, Const<2>, ArrayStorage<f32, 1, 2>>;

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

pub fn get_bezier_segments(points: &Vec<Vec2D>) -> Vec<Box<dyn Fn(f32) -> Vec2D>> {
    let (p1, p2) = get_bezier_coef(points);
    let mut i = 0usize;
    points.windows(2).map(|p| {
        let ret = get_bezier_cubic(p[0], p1[i], p2[i], p[1]);
        i += 1;
        ret
    }).collect()
}