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

pub const clover_data: [[f64; 2]; 40] = [
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

#[derive(Clone, Copy, Debug)]
pub struct WayPoint {
    segment: i32,
    offset: f64
}

impl WayPoint {
    pub fn zero() -> WayPoint {
        WayPoint {
            segment: 0,
            offset: 0.0
        }
    }
}

struct Projection {
    wp: WayPoint,
    distance: f64
}

impl Projection {
    fn project(a: Pt, b: Pt, p: Pt, segment: i32) -> Projection {
        let d = a - b;
        let mut lambda = ((a.x - p.x) * d.x + (a.y - p.y) * d.y) / (d.x*d.x + d.y*d.y);
        if lambda < 0.0 {
            lambda = 0.0;
        } else if lambda > 1.0 {
            lambda = 1.0;
        }
        let x = a + lambda * (b - a);
        let dist = (p - x).norm();
        let length = (b - a).norm();

        Projection {
            distance: dist,
            wp: WayPoint { segment: segment, offset: lambda * length}
        }
    }
}

pub struct Way {
    segment_len: Vec<f64>,
    points: Vec<Pt>,
    count: i32
}

impl Way {
    pub fn new(points0: &[[f64; 2]], scale: f64) -> Way {
        let mut points = Vec::new();
        let mut segment_len = Vec::new();
        let len = points0.len();
        for i in 0..len {
            points.push(scale * Pt::from_slice(&points0[i]));
        }
        for i in 0..len-1 {
            segment_len.push((points[i+1] - points[i]).norm());
        }
        segment_len.push((points[len - 1] - points[0]).norm());
        Way {
            count: len as i32,
            points: points,
            segment_len: segment_len
        }
    }

    pub fn where_is(&self, p: Pt) -> WayPoint {
        /*
        Нужно считать проекции на все прямые,
        проходящие через отрезки. Если проекция не попадает в отрезок, то
        берется ближайшая точка отрезка. Среди всех таких проекций выбираем ту,
        которая ближе всех к исходной точке. Если таких точек несколько, то выбираем
        любую (может последнее и неправильно, надо еще подумать).
        */
        let mut min_pr = Projection {distance: 1.0e20, wp: WayPoint::zero()};
        for i in 0..self.count {
            let a = self.points[i as usize];
            let b = self.points[if (i+1) == self.count { 0 } else { i+1 } as usize];
            let pr = Projection::project(a, b, p, i as i32);
            if pr.distance < min_pr.distance {
                min_pr = pr;
            }
        }
        return min_pr.wp;
    }

    pub fn offset(&self, old: &WayPoint, new: &WayPoint) -> f64 {
        if new.segment == old.segment {
            new.offset - old.offset
        } else if (new.segment - old.segment == 1)
               || ((new.segment == self.count - 1) && (old.segment == 0)) {
            self.segment_len[old.segment as usize] - old.offset + new.offset
        } else if (old.segment - new.segment == 1)
               || ((old.segment == self.count - 1) && (new.segment == 0)) {
            new.offset - self.segment_len[new.segment as usize] - old.offset
        } else {
            panic!("Should not be here!")
        }
    }
}
