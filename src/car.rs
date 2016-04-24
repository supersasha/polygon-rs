use geom::{self, Pt, Sect, Isx, Figure, Mtx2, recalc_rays};
use std::rc::Rc;
use std::f64::consts::PI;

pub struct Car {
    pub center: Pt,
    pub course: Pt,
    base: f64,
    length: f64,
    width: f64,
    pub wheels_angle: f64,
    pub speed: f64,
    pub rays: Vec<Sect>,
    pub path: Figure,
    walls: Rc<Figure>,
    pub isxs: Vec<Isx>,
}

impl Car {
    pub fn new(center: Pt, course: Pt, length: f64, width: f64,
                nrays: usize, walls: Rc<Figure>) -> Car {
        let mut rays = Vec::with_capacity(nrays);
        rays.resize(nrays, Sect::zero());
        let mut isxs = Vec::with_capacity(nrays);
        isxs.resize(nrays, Isx::zero());
        let path = Figure::void();
        let mut car = Car {
            center: center,
            course: course,
            length: length,
            width: width,
            base: length,
            wheels_angle: 0.0,
            speed: 0.0,
            rays: rays,
            path: path,
            walls: walls,
            isxs: isxs
        };
        car.recalc_rays();
        car.recalc_path();
        car
    }

    pub fn set_pos(&mut self, center: Pt, course: Pt) {
        self.center = center;
        self.course = course;
    }
    
    pub fn action_penalty(&self, action: &[f64]) -> f64 {
        let h = 0.1f64;
        let m = 8i32;
        let c = 5.0f64;
        let a = h / c.powi(m);
        let la = action[0].abs();
        let p = a * la.powi(m);
        let mut pp = 0.0; // extra penalty
        if la > 20.0 {
            pp = 1.0;
        }
        p / (1.0 + p.abs()) + pp
    }

    pub fn act(&mut self, action: &[f64]) {
        fn d(a: f64) -> f64 {
            a / (1.0 + a.abs())
        }
        self.speed = d(action[0]);
        self.wheels_angle = PI / 4.0 * d(action[1]);
        self.move_or_stop(0.1);
    }
    
    fn recalc_rays(&mut self) {
        geom::recalc_rays(self.rays.as_mut(), self.center, self.course);
    }

    fn recalc_path(&mut self) {
        let l = 0.5 * self.length * self.course;
        let w = rperp(self.course) * 0.5 * self.width;
        self.path = Figure::closed_path(&[self.center + l - w,
                                         self.center + l + w,
                                         self.center - l + w,
                                         self.center - l - w]);
    }

    fn move_or_stop(&mut self, dt: f64) {
        let center = self.center;
        let course = self.course;
        // TODO: check correctness: if we calculate things on right time
        self.mv(dt);
        self.recalc_path();
        //println!("path={:?}", self.path);
        let intscts = geom::figures_intersect(
            &self.path,
            &self.walls);
        //println!("intscts={}", intscts);
        if intscts {
            self.center = center;
            self.course = course;
            self.speed = 0.0;
            // TODO: check, probably we forgot to recalculate back rays
            // (surely, we should restore saved rays, not recalculate them)
            // TODO: the same with path
            self.recalc_path(); // !!!???
        } else {
            self.recalc_rays();
            geom::rays_figure_intersections(&self.rays, &self.walls,
                                           -1.0, self.isxs.as_mut())            
        }
    }

    fn mv(&mut self, dt: f64) {
        if self.wheels_angle.abs() < 0.0001 {
            self.center = self.center + self.speed * dt * self.course;
        } else {
            self.move_with_turn(dt);
        }
    }

    fn move_with_turn(&mut self, dt: f64) {
        let beta = -self.speed * dt * self.wheels_angle.tan() / self.base;
        let mut pg = Pt::zero();
        if self.wheels_angle > 0.0 {
            pg = rperp(self.course);
        } else {
            pg = lperp(self.course);
        }
        let rot_center = self.center - 0.5*self.base * self.course
            + self.base / self.wheels_angle.tan().abs() * pg;
        let s = beta.sin();
        let c = beta.cos();
        let m = Mtx2::rows(Pt::new(c, -s), Pt::new(s, c));
        self.center = rot_center + m * (self.center - rot_center);
        self.course = m * self.course;
    }
}

fn lperp(p: Pt) -> Pt {
    Pt{x: -p.y, y: p.x}
}

fn rperp(p: Pt) -> Pt {
    Pt{x: p.y, y: -p.x}
}
