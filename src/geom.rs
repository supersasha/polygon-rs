use std::{self, env};
use std::ops::{Add, Sub, Mul};

#[derive(Clone, Copy, Debug)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

impl Pt {
    pub fn zero() -> Pt {
        Pt{x: 0.0, y: 0.0}
    }
    
    pub fn new(x: f64, y: f64) -> Pt {
        Pt{x: x, y: y}
    }
}

impl Add<Pt> for Pt {
    type Output = Pt;
    fn add(self, p: Pt) -> Pt {
        Pt{x: self.x + p.x, y: self.y + p.y}
    }
}

impl Sub<Pt> for Pt {
    type Output = Pt;
    fn sub(self, p: Pt) -> Pt {
        Pt{x: self.x - p.x, y: self.y - p.y}
    }
}

impl Mul<f64> for Pt {
    type Output = Pt;
    fn mul(self, d: f64) -> Pt {
        Pt{x: self.x * d, y: self.y * d}
    }
}

impl Mul<Pt> for f64 {
    type Output = Pt;
    fn mul(self, p: Pt) -> Pt {
        Pt{x: self * p.x, y: self * p.y}
    }
}

#[derive(Clone, Copy)]
pub struct Mtx2 {
    pub rows: [Pt; 2]
}

impl Mtx2 {
    pub fn rows(p1: Pt, p2: Pt) -> Mtx2 {
        Mtx2 {
            rows: [p1, p2] 
        }
    }
}

impl Mul<Pt> for Mtx2 {
    type Output = Pt;
    fn mul(self, p: Pt) -> Pt {
        Pt {
            x: self.rows[0].x * p.x + self.rows[0].y * p.y,
            y: self.rows[1].x * p.x + self.rows[1].y * p.y
        }
    }
}


#[derive(Clone, Copy)]
pub struct Sect {
    pub p0: Pt,
    pub p1: Pt,
}

impl Sect {
    pub fn zero() -> Sect {
        Sect {
            p0: Pt::zero(),
            p1: Pt::zero()
        }
    }

    pub fn new(p0: Pt, p1: Pt) -> Sect {
        Sect {
            p0: p0,
            p1: p1
        }
    }
}

#[derive(Clone, Copy)]
pub struct Isx {
    pub point: Pt,
    pub dist: f64,
}

impl Isx {
    pub fn zero() -> Isx {
        Isx {
            point: Pt::zero(),
            dist: 0.0
        }
    }
}

pub struct Figure {
    pub sects: Vec<Sect>,
}

impl Figure {
    pub fn void() -> Figure {
        Figure {
            sects: Vec::new()
        }
    }

    pub fn closed_path(points: &[Pt]) -> Figure {
        let n = points.len();
        let mut sects = Vec::<Sect>::with_capacity(n);
        for i in 0..n-1 {
            let s = Sect::new(points[i], points[i+1]);
            sects.push(s);
        }
        sects.push(Sect::new(points[n-1], points[0]));
        Figure {
            sects: sects
        }
    }

    pub fn compound(figs: &[Figure]) -> Figure {
        let sects = figs.iter().fold(Vec::new(), |mut acc, ref f| {acc.extend(&f.sects); acc});
        Figure {
            sects: sects
        }
    }
}

//TODO: use (-) operator
fn minus_pt(p1: &Pt, p2: &Pt) -> Pt {
    Pt{x: p1.x - p2.x, y: p1.y - p2.y}
}

fn vdot(p0: &Pt, p1: &Pt) -> f64 {
    p0.x * p1.y - p0.y * p1.x
}

fn sections_intersect(subj: &Sect, obj: &Sect, is_ray: bool) -> Isx {
    let mut isx = Isx{point: Pt{x: 0.0, y: 0.0}, dist: -1.0};
    let a1 = if is_ray {
        subj.p0
    } else {
        minus_pt(&subj.p1, &subj.p0)
    };
    let a2 = minus_pt(&obj.p0, &obj.p1);
    let b =  minus_pt(&obj.p0, &subj.p0);
    let det = vdot(&a1, &a2);
    if det.abs() > 1.0e-8 {
        let x0 = vdot(&b, &a2) / det;
        let x1 = vdot(&a1, &b) / det;
        if x0 >= 0.0 && x1 >= 0.0 && x1 <= 1.0 {
            if is_ray || x0 <= 1.0 {
                isx.dist = x0;
                isx.point.x = subj.p0.x + a1.x * x0;
                isx.point.y = subj.p0.y + a1.y * x0;
            }
        }
    }
    isx
}

pub fn figures_intersect(subjs: &[Sect], objs: &[Sect]) -> bool {
    for s in subjs {
        for o in objs {
            let isx = sections_intersect(s, o, false);
            if isx.dist >= 0.0 {
                return true
            }
        }
    }
    false
}

pub fn rays_figure_intersections(rays: &[Sect],
                             figure: &[Sect],
                             infinity: f64,
                             intersections: &mut[Isx]) {

    for (i, r) in rays.iter().enumerate() {
        let mut min_isx = Isx{point: Pt{x: 0.0, y: 0.0}, dist: 1.0e20};
        for p in figure {
            let isx = sections_intersect(r, p, true);
            if isx.dist >= 0.0 && isx.dist < min_isx.dist {
                min_isx = isx;
            }
        }
        if min_isx.dist < 0.0 {
            intersections[i].dist = infinity;
        } else {
            intersections[i] = min_isx;
        }
    }
}

pub fn recalc_rays(rays: &mut[Sect], center: Pt, course: Pt) {
    let k = 2.0 * std::f64::consts::PI / (rays.len() as f64);
    let rn = rays.len();
    for i in 0..rn {
        let angle = k * (i as f64);
        let s = angle.sin();
        let c = angle.cos();
        rays[i] = Sect{p0: center,
                       p1: Pt{x: c * course.x - s * course.y,
                              y: s * course.x + c * course.y}}
    }
}
/*
fn perftest(f: i32) {
    let subj = Sect{p0: Pt{x: 2.0, y: 2.0}, p1: Pt{x: 0.5, y: 0.5}};
    let subj_prim = Sect{p0: Pt{x: 1.9, y: 2.1}, p1: Pt{x: 0.5, y: 0.5}};
    let obj = Sect{p0: Pt{x: 0.0, y: 1.0}, p1: Pt{x: 1.0, y: 1.0}};
    let mut r = 0.0f64;
    for i in 0..1000000001 {
        if ((f + i) & 3) != 0 {
            r += sections_intersect(&subj, &obj, false).dist
        } else {
            r += sections_intersect(&subj_prim, &obj, false).dist
        };
    };
    println!("R = {}", r);
}

fn main() {
    let f: i32 = env::args().nth(1).unwrap().parse().unwrap_or(0i32);
    
    perftest(f);
}
*/
