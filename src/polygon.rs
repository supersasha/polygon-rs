use std::rc::Rc;
use std::cell::RefCell;
use car::Car;
use geom::{Figure, Pt};
use track::{clover};
use cacla::{Cacla, Range};
use std::f64::consts::PI;

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
    pub car: Rc<RefCell<Car>>,
    pub walls: Rc<Figure>,
    pub state: Vec<f64>,
    //pub prev_state: Vec<f64>,
    pub last_action: Vec<f64>
}

impl World {
    pub fn new(car: Rc<RefCell<Car>>, walls: Rc<Figure>,
           state_dim: usize, action_dim: usize) -> World {
        let mut state = Vec::with_capacity(state_dim);
        state.resize(state_dim, 0.0);
        let mut last_action = Vec::with_capacity(action_dim);
        last_action.resize(action_dim, 0.0);
        World {
            car: car,
            walls: walls,
            state: state,
            //prev_state: state.clone(),
            last_action: last_action
        }
    }
    
    pub fn act(&mut self, action: &[f64]) {
        self.car.borrow_mut().act(action);
        self.recalc_state();
        self.last_action.clone_from_slice(action);
    }

    pub fn reward(&self) -> f64 {
        let mut hp: f64 = 0.0;
        if self.car.borrow().speed.abs() < 0.001 {
            hp = 1.0;
        }
        let ap = 2.0 * self.car.borrow().wheels_angle.abs();
        let cap = self.car.borrow().action_penalty(&self.last_action);
        let penalty = ap + hp + cap;
        let speed = self.car.borrow().speed;
        if speed > 0.0 {
            speed - penalty
        } else {
            -0.5 * speed - penalty
        }
    }
    
    fn recalc_state(&mut self) {
        //self.prev_state.clone_from(&self.state);
        let n = self.car.borrow().isxs.len(); 
        for (i, isx) in self.car.borrow().isxs.iter().enumerate() {
            self.state[i] = isx.dist;
        }
        self.state[n] = self.car.borrow().speed;
        self.state[n+1] = self.car.borrow().wheels_angle;
    }
}

pub struct Polygon {
    pub world: Rc<RefCell<World>>,
    pub car: Rc<RefCell<Car>>,
    pub walls: Rc<Figure>,
    pub last_reward: f64,
    learner: Rc<RefCell<Cacla>>,
    minmax: MinMax,
    reward_range: Range,
}

impl Polygon {
    pub fn new() -> Polygon {
        let nrays = 36;
        let walls = Rc::new(clover(2.0, 10.0));
        let action_dim = 2;
        let state_dim = nrays + 2;
        let car = Rc::new(RefCell::new(Car::new(Pt::new(-110.0, 0.0),
                                        Pt::new(0.0, 1.0),
                                        3.0, // length
                                        1.6, // width
                                        nrays,
                                        walls.clone())));
        let world = Rc::new(RefCell::new(
                            World::new(car.clone(), walls.clone(), state_dim, action_dim
                        )));
        let state_ranges = Polygon::mk_state_ranges(state_dim);
        let minmax = MinMax::new(&state_ranges);
        let learner = Rc::new(RefCell::new(Cacla::new(&state_ranges,
                            action_dim as u32,
                            200,   // hidden
                            0.99,  // gamma
                            0.01,  // alpha
                            0.001, // beta
                            0.1)));  // sigma
                        
        Polygon {
            world: world.clone(),
            car: car.clone(),
            walls: walls.clone(),
            learner: learner.clone(),
            minmax: minmax,
            reward_range: Range::new(-4.0, 1.0),
            last_reward: 0.0
        }        
    }
    
    pub fn run(&mut self, ncycles: u32) {
        let mut s = self.world.borrow().state.clone();
        let mut new_s = self.world.borrow().state.clone();
        for _ in 0..ncycles {
            self.minmax.norm(self.world.borrow().state.as_ref(), s.as_mut());
            let a = self.learner.borrow().get_action(self.world.borrow().state.as_ref());
            self.world.borrow_mut().act(a.borrow().as_ref());
            let r = self.world.borrow().reward();
            self.last_reward = r;
            self.minmax.norm(self.world.borrow().state.as_ref(), new_s.as_mut());
            self.learner.borrow_mut().step(s.as_ref(),
                                    new_s.as_ref(),
                                    a.borrow().as_ref(),
                                    normalize(&self.reward_range, r, &TRANGE));
            /*if i % 10000 == 0 {
                println!("{:?} {:?} {:?} {:?}",
                        i,
                        a.borrow(),
                        world.car.borrow().center,
                        world.car.borrow().speed);
            }*/
        }        
    }
    
    fn mk_state_ranges(state_dim: usize) -> Vec<Range> {
        let mut state_ranges = Vec::new();
        state_ranges.resize(state_dim, Range::zero());
        for i in 0..36 {
            state_ranges[i] = Range::new(0.0, 100.0);
        }
        state_ranges[36] = Range::new(-1.0, 1.0);
        state_ranges[37] = Range::new(-PI/4.0, PI/4.0);
        state_ranges        
    }
}

// TODO: remove when sure all works
/*
pub fn run(ncycles: usize) {
    let walls = Rc::new(clover(2.0, 10.0));
    //println!("Walls: {:?}", walls);
    let nrays = 36;
    let action_dim = 2;
    let state_dim = nrays + 2;
    let car = Rc::new(RefCell::new(Car::new(Pt::new(-110.0, 0.0),
                                    Pt::new(0.0, 1.0),
                                    3.0, // length
                                    1.6, // width
                                    nrays,
                                    walls.clone())));

    let mut world = World::new(car, walls.clone(), state_dim, action_dim);

    let mut state_ranges = Vec::new();
    state_ranges.resize(state_dim, Range::zero());
    for i in 0..36 {
        state_ranges[i] = Range::new(0.0, 100.0);
    }
    state_ranges[36] = Range::new(-1.0, 1.0);
    state_ranges[37] = Range::new(-PI/4.0, PI/4.0);

    let minmax = MinMax::new(&state_ranges);
    let learner = RefCell::new(Cacla::new(&state_ranges,
                                action_dim as u32,
                                100,   // hidden
                                0.99,  // gamma
                                0.01,  // alpha
                                0.001, // beta
                                0.1));  // sigma
    
    let mut s = world.state.clone();
    let mut new_s = world.state.clone();
    let reward_range = Range::new(-4.0, 1.0);
    for i in 0..ncycles {
        minmax.norm(world.state.as_ref(), s.as_mut());
        let a = learner.borrow().get_action(world.state.as_ref());
        world.act(a.borrow().as_ref());
        let r = world.reward();
        minmax.norm(world.state.as_ref(), new_s.as_mut());
        learner.borrow_mut().step(s.as_ref(),
                                  new_s.as_ref(),
                                  a.borrow().as_ref(),
                                  normalize(&reward_range, r, &TRANGE));
        if i % 10000 == 0 {
            println!("{:?} {:?} {:?} {:?}",
                     i,
                     a.borrow(),
                     world.car.borrow().center,
                     world.car.borrow().speed);
        }
    }
}
*/