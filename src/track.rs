use geom::{Figure, Sect, Pt};

pub fn clover(d: f64, scale: f64) -> Figure {
    make_track(&clover_data, d, scale)
}

pub fn make_track(points0: &[[f64; 2]], d: f64, scale: f64) -> Figure {
    let points = points0.iter()
        .map(|p| scale * Pt::new(p[0], p[1]))
        .collect::<Vec<Pt>>();
    let n = points.len();
    //println!("points={:?}", points);
    let mut ps1: Vec<Pt> = Vec::with_capacity(n);
    let mut ps2: Vec<Pt> = Vec::with_capacity(n);
    for i in 0..n {
        let x0 = if i > 1 { points[i - 2] } else { points[n-2+i] };
        let x1 = if i > 0 { points[i - 1]} else { points[n-1+i] };
        let x2 = points[i];
        let y1 = normalized(x1 - x0);
        let y2 = normalized(x1 - x2);
        let y = normalized(y1 + y2);
        let s = vec_prod_sign(y1, y);
        let z1 = x1 + s*d*y;
        let z2 = x1 - s*d*y;
        //println!("x1={:?}, x2={:?}, y2={:?}, s={:?}, z1={:?}", x1, x2, y2, s, z1);
        ps1.push(z1);
        ps2.push(z2);
    }
    let f1 = Figure::closed_path(ps1.as_ref());
    let f2 = Figure::closed_path(ps2.as_ref());
    Figure::compound(&[f1, f2])
}

fn normalized(v: Pt) -> Pt {
    1.0 / (v.x*v.x + v.y*v.y).sqrt() * v
}

fn vec_prod_sign(v1: Pt, v2: Pt) -> f64 {
    let v = v1.x*v2.y - v1.y*v2.x;
    v.signum()
}

const clover_data: [[f64; 2]; 40] = [
            [-11.0, 1.0],
            [-9.0, 3.0],
            [-7.0, 3.0],
            [-5.0, 1.0],
            [-3.0, 1.0],
            [-1.0, 3.0],
            [-1.0, 5.0],
            [-3.0, 7.0],
            [-3.0, 9.0],
            [-1.0, 11.0],
            [1.0, 11.0],
            [3.0, 9.0],
            [3.0, 7.0],
            [1.0, 5.0],
            [1.0, 3.0],
            [3.0, 1.0],
            [5.0, 1.0],
            [7.0, 3.0],
            [9.0, 3.0],
            [11.0, 1.0],
            [11.0, -1.0],
            [9.0, -3.0],
            [7.0, -3.0],
            [5.0, -1.0],
            [3.0, -1.0],
            [1.0, -3.0],
            [1.0, -5.0],
            [3.0, -7.0],
            [3.0, -9.0],
            [1.0, -11.0],
            [-1.0, -11.0],
            [-3.0, -9.0],
            [-3.0, -7.0],
            [-1.0, -5.0],
            [-1.0, -3.0],
            [-3.0, -1.0],
            [-5.0, -1.0],
            [-7.0, -3.0],
            [-9.0, -3.0],
            [-11.0, -1.0]
        ];
