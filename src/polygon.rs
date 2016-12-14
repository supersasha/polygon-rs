use std::rc::Rc;
use std::cell::RefCell;
use car::Car;
use geom::{Figure, Pt};
use track::{clover, Way, WayPoint, clover_data};
use cacla::{Cacla, Range};
use std::f64::consts::PI;
use std::path;
use rand::{thread_rng, Rng};

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
    fn norm(&self, inp: &Vec<f64>, out: &mut Vec<f64>) {
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

    pub fn clone(&self) -> World {
        World {
            car: self.car.clone(),
            walls: self.walls.clone(),
            way: self.way.clone(),
            way_point: self.way_point,
            old_way_point: self.old_way_point,
            state: self.state.clone(),
            last_action: self.last_action.clone()
        }
    }

    pub fn act(&mut self, action: &Vec<f64>) {
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

        //println!("--reward");
        //println!("center: {:?}", self.car.center);
        //println!("course: {:?}", self.car.course);
        //println!("speed: {:?}", self.car.speed);
        //println!("wheels: {:?}", self.car.wheels_angle);
        //println!("action: {:?}", self.last_action);
        let offset = self.way.offset(&self.old_way_point, &self.way_point);
        let offset_reward = 300.0 * offset;
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
        let speed_penalty = -speed*speed;
        let mut speed_reward = speed;
        if speed < 0.0 {
            speed_reward = -speed/2.0;
        }

        /*offset_reward +*/ 10.0*speed_reward + /*speed_reward +*/ 20.0*dist_reward + 5.0*wheels_reward + action_reward + 10.0 * speed_penalty
    }

    fn recalc_state(&mut self) {
        //self.prev_state.clone_from(&self.state);
        let n = self.nrays();
        for (i, isx) in self.car.isxs.iter().enumerate() {
            self.state[i] = if isx.dist < 10.0 { isx.dist } else { 10.0 }; // 10.0; // !!!
        }
        //self.state[n] = self.car.speed; // / 1.0; // !!!
        //self.state[n+1] = self.car.wheels_angle; // / 1.0; // !!!
        //self.state[n+2] = self.car.action_penalty3(&self.last_action);
        
        //println!("--recalc_state");
        //println!("center: {:?}", self.car.center);
        //println!("course: {:?}", self.car.course);
        //println!("speed: {:?}", self.car.speed);
        //println!("wheels: {:?}", self.car.wheels_angle);
        //println!("action: {:?}", self.last_action);
        
        //self.state[n+3] = self.way.offset(&self.old_way_point, &self.way_point);
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
    pub worlds: Vec<World>,
    pub walls: Rc<Figure>,
    pub last_reward: f64,
    pub learner: Cacla,
    minmax: MinMax,
    reward_range: Range,
    stopped_cycles: u32,
    wander_cycles: u32,
    epoch: u32,
    ws_dir: path::PathBuf,
    current_index: usize,
}

impl Polygon {
    pub fn new(ws_dir: path::PathBuf) -> Polygon {
        let nrays = 36;
        let scale = 10.0;
        let walls = Rc::new(clover(4.0, scale));
        let way = Rc::new(Way::new(&clover_data, scale));
        let action_dim = 2;
        let state_dim = nrays; //+ 4; // speed + angle + action_penalty + offset
        let world = World::new(nrays,
                                walls.clone(),
                                way.clone(),
                                state_dim,
                                action_dim);
        let state_ranges = Polygon::mk_state_ranges(state_dim);
        let minmax = MinMax::new(&state_ranges);
        let learner = Cacla::new(&state_ranges,
                            action_dim as u32,
                            18,   // hidden
                            0.99,  // gamma
                            0.1, // alpha !!!
                            0.001, // beta
                            0.1);  // sigma
        let mut worlds = Vec::with_capacity(20);
        for i in 0..worlds.capacity() {
            let mut w = world.clone();
            //let angle = PI/4.0 * (i as f64 / worlds.capacity() as f64);
            //w.car.course = Pt::new(angle.cos(), angle.sin());
            worlds.push(w);
        }

        Polygon {
            worlds: worlds,
            walls: walls.clone(),
            learner: learner,
            minmax: minmax,
            reward_range: Range::new(-100.0, 100.0), //(-4.0, 1.0) - for reward_old,
            last_reward: 0.0,
            stopped_cycles: 0,
            wander_cycles: 0,
            epoch: 1000000,
            ws_dir: ws_dir,
            current_index: 0,
        }
    }

    pub fn save(&self) {
        self.learner.save(&self.ws_dir);
    }

    pub fn load(&mut self) {
        self.learner.load(&self.ws_dir);
    }

    pub fn run(&mut self, ncycles: u32) -> f64 {
        let mut s = self.worlds[0].state.clone();
        let mut new_s = self.worlds[0].state.clone();
        let N = self.worlds.capacity();
        let mut sum_reward = 0.0;
        for _ in 0..ncycles {
            sum_reward += self.run_once_for_world(0, &mut s, &mut new_s);
            for i in 1..N {
                self.run_once_for_world(i, &mut s, &mut new_s);
            }
        }
        /*
        let M = 20;
        let mut rng = thread_rng();
        for _ in 0..ncycles {
            if self.worlds.len() > N - 2 {
                for i in 0..M {
                    let index = rng.gen_range(0, self.worlds.len());
                    self.run_once_for_world(index, &mut s, &mut new_s);
                }
            }
            if self.current_index < N - 1 {
                if self.worlds.len() < N {
                    let new_world = self.worlds[self.current_index].clone();
                    self.worlds.push(new_world);
                    self.current_index += 1;
                } else {
                    //if rng.gen_range(0, 10) == 0 {
                    {
                        let new_world = self.worlds[self.current_index].clone();
                        self.worlds[self.current_index + 1] = new_world;
                        self.current_index += 1;
                    }
                }
            } else {
                self.worlds[0] = self.worlds[self.current_index].clone();
                self.current_index = 0;
            }
            let idx = self.current_index;
            sum_reward += self.run_once_for_world(idx, &mut s, &mut new_s);
        }
        */
        sum_reward
    }

    pub fn run_once_for_world(&mut self, index: usize, s: &mut Vec<f64>, new_s: &mut Vec<f64>) -> f64 {
        self.minmax.norm(&self.worlds[index].state, s);
        let a = self.learner.get_action(s, false);
        self.worlds[index].act(&a);
        //println!("state(0): {:?}\n-----------------------------------", &self.worlds[index].state);
        let r = self.worlds[index].reward();

        self.minmax.norm(&self.worlds[index].state, new_s);
        let mut i = 0;
        let new_s_ = new_s.clone();
        for x in new_s.into_iter() {
            if *x > 0.9 || *x < -0.9 {
                println!("x[{}]={}", i, x);
                println!("new_s: {:?}", &new_s_);
                println!("state: {:?}", &self.worlds[index].state);
                panic!("Normalized out of range");
            }
            i += 1;
        }
        self.learner.step(s, new_s, &a,
                            normalize(&self.reward_range, r, &TRANGE));
        self.last_reward = r;
        r
    }

    pub fn current_world(&self) -> &World {
        &self.worlds[self.current_index]
    }

    pub fn get_world(&self, idx: usize) -> &World {
        &self.worlds[idx]
    }

    pub fn get_worlds_size(&self) -> usize {
        self.worlds.len()
    }

    pub fn v_fn(&self, n: u32) -> Box<Fn(f64) -> f64> {
        let s = self.current_world().state.clone();
        let f = self.learner.v_fn();
        Box::new(move |x| {
            let mut state = s.clone();
            state[n as usize] = x;
            let y = f(&state);
            y[0]
        })
    }

    pub fn ac_fn(&self, n: u32, m: u32) -> Box<Fn(f64) -> f64> {
        let s = self.current_world().state.clone();
        let f = self.learner.ac_fn();
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
        for i in 0..state_dim {
            state_ranges[i] = Range::new(-5.0, 20.0);
        }
        //state_ranges[state_dim-1] = Range::new(-10.0, 10.0); //Range::new(-1.0, 1.0);       // speed
        //state_ranges[state_dim-1] = Range::new(-10.0, 10.0); //Range::new(-PI/4.0, PI/4.0); // angle
        //state_ranges[state_dim-1] = Range::new(-10.0, 100.0); //Range::new(0.0, 100.0);     // action penalty
        //state_ranges[state_dim-1] = Range::new(-10.0, 10.0); //Range::new(-2.0, 2.0);       // offset
        state_ranges
    }
}