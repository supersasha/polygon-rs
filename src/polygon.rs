use std::rc::Rc;
use std::cell::RefCell;
use car::Car;
use geom::{Figure, Pt};
use track::{clover, Way, WayPoint, clover_data};
use cacla::{Cacla, Range};
use std::f64::consts::PI;
use std::path;

const TRANGE: Range = Range{lo: -1.0, hi: 1.0};

pub struct MinMax {
    ranges: Vec<Range>,
}

impl MinMax {
    fn new(ranges: &Vec<Range>) -> MinMax {
        MinMax {
            ranges: ranges.clone()
        }
    }
    fn norm(&self, inp: &[f64], out: &mut [f64]) {
        let n = inp.len();
        for i in 0..n {
            out[i] = normalize(&self.ranges[i], inp[i], &TRANGE);
        }
        //println!("rng={:?}", self.ranges);
    }
    /*
    fn denorm(&self, inp: &[f64], out: &mut [f64]) {
        let n = inp.len();
        for i in 0..n {
            out[i] = normalize(&TRANGE, inp[i], &self.ranges[i]);
        }
    }
    */
}

fn normalize(from: &Range, x: f64, to: &Range) -> f64 {
    to.lo + (x - from.lo) * (to.hi - to.lo) / (from.hi - from.lo)
}

pub struct World {
    pub car: Car,
    pub walls: Rc<Figure>,
    pub way: Rc<Way>,
    pub way_point: WayPoint,
    pub old_way_point: WayPoint,
    pub state: Vec<f64>,
    //pub prev_state: Vec<f64>,
    pub last_action: Vec<f64>
}

impl World {
    pub fn new(nrays: usize, walls: Rc<Figure>,
           way: Rc<Way>,
           state_dim: usize, action_dim: usize) -> World {
        let mut state = Vec::with_capacity(state_dim);
        state.resize(state_dim, 0.0);
        let mut last_action = Vec::with_capacity(action_dim);
        last_action.resize(action_dim, 0.0);
        let car = Car::new(Pt::new(-110.0, 0.0),
                            Pt::new(0.0, 1.0),
                            3.0, // length
                            1.6, // width
                            nrays,
                            walls.clone());
        let center = car.center;
        World {
            car: car,
            walls: walls,
            way: way.clone(),
            way_point: way.where_is(center),
            old_way_point: WayPoint::zero(),
            state: state,
            //prev_state: state.clone(),
            last_action: last_action
        }
    }

    pub fn act(&mut self, action: &[f64]) {
        self.car.act(action);
        self.old_way_point = self.way_point;
        self.way_point = self.way.where_is(self.car.center);
        self.recalc_state();
        self.last_action.clone_from_slice(action);
    }

    pub fn reward_old_2(&self) -> f64 {
        let mut hp: f64 = 0.0;
        if self.car.speed.abs() < 0.001 {
            hp = 1.0;
        }
        let ap = 2.0 * self.car.wheels_angle.abs();
        let cap = self.car.action_penalty(&self.last_action);
        let penalty = ap + hp + cap;
        let speed = self.car.speed;
        if speed > 0.0 {
            speed - penalty
        } else {
            -0.5 * speed - penalty
        }
    }

    pub fn reward_old(&self) -> f64 {
        /*
        let mut hit_penalty = 0.0;
        if self.car.borrow().speed.abs() < 0.0000001 {
            hit_penalty = 0.1;
        }

        TODO: New continuous hit penalty
        */
        let mut min_dist = 1.0e20;
        for i in 0..self.state.len() - 2 {
            if (self.state[i] >= 0.0) && (self.state[i] < min_dist) {
                min_dist = self.state[i];
            }
        }

        //let hit_penalty = if min_dist < 1.0 { 1.0 - min_dist } else { 0.0 };

        let sigma = 2.0 / 3.0;
        let hit_penalty = (-min_dist*min_dist / (2.0 * sigma * sigma)).exp();

        let ap = self.car.wheels_angle.abs(); // 0.0..2.0
        let cap = self.car.action_penalty(&self.last_action); // 0.0..2.0
        let cap2 = self.car.action_penalty2(&self.last_action); // 0.0..1.0
        //let penalty = hit_penalty + cap + ap;

        let penalty = 1.0 * (1.0 * hit_penalty + 1.0 * cap + 1.0 * ap + 2.0 * cap2);
        //let reward = self.way.offset(&self.old_way_point, &self.way_point); // -1.0..1.0
        //5.0 * reward - penalty

        let speed = self.car.speed;
        if speed > 0.0 {
            speed - penalty
        } else {
            -0.5 * speed - penalty
        }

    }

    pub fn reward(&self) -> f64 {
        let speed = self.car.speed;
        let mut dist_reward = 0.0;
        let speed_reward = 1.0 - (speed - 1.0) * (speed - 1.0);
        /*let mut speed_reward = speed;
        if speed < 0.0 {
            speed_reward /= 2.0;
        }*/

        for i in 0..self.nrays() {
            let s = self.state[i];
            dist_reward = min(s * (1.0 - 0.0099 * s), dist_reward);
        }

        let wheels = self.car.wheels_angle;
        let wheels_reward = - wheels * wheels;

        let action_penalty = self.car.action_penalty3(&self.last_action);
        let action_reward = -action_penalty * action_penalty;

        speed_reward + 20.0*dist_reward + 10.0*wheels_reward + action_reward
    }

    fn recalc_state(&mut self) {
        //self.prev_state.clone_from(&self.state);
        let n = self.nrays();
        for (i, isx) in self.car.isxs.iter().enumerate() {
            self.state[i] = if isx.dist >= 0.0 { isx.dist } else { 10.0 }; // 10.0; // !!!
        }
        self.state[n] = self.car.speed; // / 1.0; // !!!
        self.state[n+1] = self.car.wheels_angle; // / 1.0; // !!!
        self.state[n+2] = self.car.action_penalty3(&self.last_action);
    }

    fn nrays(&self) -> usize {
        self.car.isxs.len()
    }
}

fn min(a: f64, b: f64) -> f64 {
    if a > b {
        b
    } else {
        a
    }
}

pub struct Polygon {
    pub world: World,
    pub walls: Rc<Figure>,
    pub last_reward: f64,
    pub learner: Rc<RefCell<Cacla>>,
    minmax: MinMax,
    reward_range: Range,
    stopped_cycles: u32,
    wander_cycles: u32,
    epoch: u32,
    ws_dir: path::PathBuf
}

impl Polygon {
    pub fn new(ws_dir: path::PathBuf) -> Polygon {
        let nrays = 36;
        let scale = 10.0;
        let walls = Rc::new(clover(2.0, scale));
        let way = Rc::new(Way::new(&clover_data, scale));
        let action_dim = 2;
        let state_dim = nrays + 2 + 1;
        let world = World::new(nrays,
                                walls.clone(),
                                way.clone(),
                                state_dim,
                                action_dim);
        let state_ranges = Polygon::mk_state_ranges(state_dim);
        let minmax = MinMax::new(&state_ranges);
        let learner = Rc::new(RefCell::new(Cacla::new(&state_ranges,
                            action_dim as u32,
                            30,   // hidden
                            0.99,  // gamma
                            0.01, // alpha !!!
                            0.001, // beta
                            0.1)));  // sigma

        Polygon {
            world: world,
            walls: walls.clone(),
            learner: learner.clone(),
            minmax: minmax,
            reward_range: Range::new(-300.0, 40.0), //(-4.0, 1.0) - for reward_old,
            last_reward: 0.0,
            stopped_cycles: 0,
            wander_cycles: 0,
            epoch: 1000000,
            ws_dir: ws_dir
        }
    }

    pub fn save(&self) {
        self.learner.borrow().save(&self.ws_dir);
    }

    pub fn load(&mut self) {
        self.learner.borrow_mut().load(&self.ws_dir);
    }

    pub fn run(&mut self, ncycles: u32) {
        let mut s = self.world.state.clone();
        let mut new_s = self.world.state.clone();
        for _ in 0..ncycles {
            self.minmax.norm(self.world.state.as_ref(), s.as_mut());
            let a = self.learner.borrow().get_action(
                s.as_ref(), false);
            self.world.act(a.borrow().as_ref());
            let r = self.world.reward();

            self.minmax.norm(self.world.state.as_ref(), new_s.as_mut());
            self.learner.borrow_mut().step(s.as_ref(),
                                    new_s.as_ref(),
                                    a.borrow().as_ref(),
                                    normalize(&self.reward_range, r, &TRANGE));
            self.last_reward = r;
        }
    }

    pub fn v_fn(&self, n: u32) -> Box<Fn(f64) -> f64> {
        let s = self.world.state.clone();
        let l = self.learner.clone();
        let f = l.borrow().v_fn();
        Box::new(move |x| {
            let mut state = s.clone();
            state[n as usize] = x;
            let y = f(&state);
            y[0]
        })
    }

    pub fn ac_fn(&self, n: u32, m: u32) -> Box<Fn(f64) -> f64> {
        let s = self.world.state.clone();
        let l = self.learner.clone();
        let f = l.borrow().ac_fn();
        Box::new(move |x| {
            let mut state = s.clone();
            state[n as usize] = x;
            let y = f(&state);
            y[m as usize]
        })
    }

    fn mk_state_ranges(state_dim: usize) -> Vec<Range> {
        let mut state_ranges = Vec::new();
        state_ranges.resize(state_dim, Range::zero());
        for i in 0..state_dim-2 {
            state_ranges[i] = Range::new(0.0, 10.0);
        }
        state_ranges[state_dim-3] = Range::new(-1.0, 1.0);
        state_ranges[state_dim-2] = Range::new(-PI/4.0, PI/4.0);
        state_ranges[state_dim-1] = Range::new(0.0, 100.0);
        state_ranges
    }
}