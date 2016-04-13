use std::rc::Rc;
use std::cell::RefCell;
use car::Car;
use geom::{Figure, Pt};
use track::{clover};
use cacla::{Cacla, Range};

pub struct Polygon {
    pub car: RefCell<Car>,
    pub walls: Rc<Figure>,
    pub state: Vec<f64>,
    pub prev_state: Vec<f64>,
    last_action: Vec<f64>,
}

impl Polygon {
    fn new(car: RefCell<Car>, walls: Rc<Figure>,
           state_dim: usize, action_dim: usize) -> Polygon {
        let mut state = Vec::with_capacity(state_dim);
        state.resize(state_dim, 0.0);
        let mut last_action = Vec::with_capacity(action_dim);
        last_action.resize(action_dim, 0.0);
        Polygon {
            car: car,
            walls: walls,
            state: state,
            prev_state: Vec::with_capacity(state_dim),
            last_action: last_action
        }
    }
    
    fn act(&mut self, action: &[f64]) {
        self.car.borrow_mut().act(action);
        self.recalc_state();
        self.last_action.clone_from_slice(action);
    }

    fn reward(&self) -> f64 {
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
        // TODO
        self.prev_state.clone_from(&self.state);
        let n = self.car.borrow().isxs.len(); 
        for (i, isx) in self.car.borrow().isxs.iter().enumerate() {
            self.state[i] = isx.dist;
        }
        self.state[n] = self.car.borrow().speed;
        self.state[n+1] = self.car.borrow().wheels_angle;
    }
}

pub fn run(ncycles: usize) {
    let walls = Rc::new(clover(2.0, 10.0));
    let nrays = 36;
    let action_dim = 2;
    let state_dim = nrays + 2;
    let car = RefCell::new(Car::new(Pt::new(-110.0, 0.0),
                                    Pt::new(0.0, 1.0),
                                    3.0, // length
                                    1.6, // width
                                    nrays,
                                    walls.clone()));

    let mut world = Polygon::new(car, walls.clone(), state_dim, action_dim);

    let mut state_ranges = Vec::new();
    state_ranges.resize(state_dim, Range::new(-1.0, 1.0));
    let learner = RefCell::new(Cacla::new(&state_ranges,
                                action_dim as u32,
                                100,   // hidden
                                0.99,  // gamma
                                0.01,  // alpha
                                0.001, // beta
                                0.1));  // sigma
    
    // TODO: complete; normalize state, action
    for i in 0..ncycles {
        let a = learner.borrow().get_action(world.state.as_ref());
        world.act(a.borrow().as_ref());
        let r = world.reward();
        learner.borrow_mut().step(world.prev_state.as_ref(),
                                  world.state.as_ref(),
                                  a.borrow().as_ref(), r);
    }
}
