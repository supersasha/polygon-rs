//#![feature(custom_derive, plugin)]
use fann::{Fann, FannType, ActivationFunc, TrainAlgorithm, IncrementalParams};
use rand::distributions::{Normal, IndependentSample};
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::ops::DerefMut;
use rand;
use rustc_serialize::json;
use std::fs::File;
use std::io::prelude::*;
use std::path;

#[derive(Clone, Debug)]
pub struct Range {
    pub lo: FannType,
    pub hi: FannType,
}

impl Range {
    pub fn new(lo: FannType, hi: FannType) -> Range {
        Range {
            lo: lo,
            hi: hi
        }
    }

    pub fn zero() -> Range {
        Range {
            lo: 0.0,
            hi: 0.0
        }
    }
}

struct Approx {
    net: Fann,
    ranges: Vec<Range>,
    hidden: u32,
    output: u32,
}

impl Approx {
    fn new(ranges: &Vec<Range>, hidden: u32, output: u32, learning_rate: f64) -> Approx {
        //let mut rs = Vec::with_capacity();
        //rs.clone_from_slice(ranges);
        let rs = ranges.to_vec();
        let mut net = Fann::new(&[ranges.len() as u32, hidden, output]).unwrap();
        //TODO:adjust network params
        net.set_activation_func_hidden(ActivationFunc::SigmoidSymmetric);
        net.set_activation_func_output(ActivationFunc::Linear);
        net.randomize_weights(-0.001, 0.001);
        let train_params = IncrementalParams{learning_momentum: 0.0,
                                             learning_rate: learning_rate as f32};
        net.set_train_algorithm(TrainAlgorithm::Incremental(train_params));
        Approx {
            ranges: rs,
            hidden: hidden,
            output: output,
            net: net,
        }
    }

    fn call(&self, x: &[FannType]) -> Vec<FannType> {
        // TODO: optimize!!!
        self.net.run(x).unwrap()
    }

    fn update(&mut self, target: &[FannType], x: &[FannType]) {
        self.net.train(x, target);
    }

    fn save(&self, filename: &path::PathBuf) {
        self.net.save(filename);
    }

    fn load(&mut self, filename: &path::PathBuf) {
        // TODO: save and load other settings
        self.net = Fann::from_file(filename).unwrap()
    }

    fn print(&self) {
        self.net.print_connections();
    }
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct CaclaState {
    action: Rc<RefCell<Vec<f64>>>,
    alpha: f64,
    beta: f64,
    gamma: f64,
    pub sigma: RefCell<f64>,
    var: f64,
}

pub struct Cacla {
    V: Rc<RefCell<Approx>>,
    Ac: Rc<RefCell<Approx>>,
    pub state: CaclaState,
}

impl Cacla {
    pub fn new(state_ranges: &Vec<Range>,
           dim_actions: u32, hidden: u32,
           gamma: f64,
           alpha: f64,
           beta: f64,
           sigma: f64) -> Cacla {
        let mut action = Vec::with_capacity(dim_actions as usize);
        action.resize(dim_actions as usize, 0.0);
        Cacla {
            state: CaclaState {
                alpha: alpha,
                beta: beta,
                gamma: gamma,
                sigma: RefCell::new(sigma),
                var: 1.0,
                action: Rc::new(RefCell::new(action))
            },
            V: Rc::new(RefCell::new(Approx::new(state_ranges, hidden, 1, alpha))),
            Ac: Rc::new(RefCell::new(Approx::new(state_ranges, hidden, dim_actions, alpha)))
        }
    }

    pub fn get_action(&self, state: &[FannType], wander_more: bool) -> Rc<RefCell<Vec<f64>>> {
        let mu = self.Ac.borrow().call(state);
        let mut rng = rand::thread_rng();
        let mut action = self.state.action.borrow_mut();
        let mut sigma = self.state.sigma.borrow_mut();
        //if wander_more {
        //    sigma = 1.0
        //}
        for i in 0..mu.len() {
            let normal = Normal::new(mu[i], *sigma.deref_mut());
            action[i] = normal.ind_sample(&mut rng);
        }
        if *sigma.deref_mut() > 0.1 {
            *sigma.deref_mut() *= 0.99999993068528434627048314517621;
        }
        self.state.action.clone()
    }

    pub fn step(&mut self, old_state: &[FannType], new_state: &[FannType],
            action: &[FannType], reward: f64) {
        let old_state_v = self.V.borrow().call(old_state);
        let new_state_v = self.V.borrow().call(new_state);
        let target = &[reward + self.state.gamma * new_state_v[0]];
        let td_error = target[0] - old_state_v[0];
        self.V.borrow_mut().update(target, old_state);
        if td_error > 0.0 {
            self.state.var = (1.0 - self.state.beta) * self.state.var
                            + self.state.beta * td_error * td_error;
            let n = (td_error / self.state.var.sqrt()).ceil() as usize;
            // TODO: print n if n > 5
            if n > 5 {
                println!("n = {}", n);
            }
            for _ in 0..n {
                self.Ac.borrow_mut().update(action, old_state)
            }
        }
    }

    pub fn save(&self, dir: &path::PathBuf) {
        let mut f = File::create(&dir.join("/cacla.state")).unwrap();
        write!(f, "{}", json::encode(&self.state).unwrap());
        self.V.borrow().save(&dir.join("/V.net"));
        self.Ac.borrow().save(&dir.join("/Ac.net"));
    }

    pub fn load(&mut self, dir: &path::PathBuf) {
        let mut f = File::open(&dir.join("/cacla.state")).unwrap();
        let mut js = String::new();
        f.read_to_string(&mut js);
        self.state = json::decode(&js).unwrap();
        self.V.borrow_mut().load(&dir.join("/V.net"));
        self.Ac.borrow_mut().load(&dir.join("/Ac.net"));
    }

    pub fn v_fn(&self) -> Box<Fn(&[FannType]) -> Vec<FannType>> {
        let V = self.V.clone();
        Box::new(move |x| V.borrow().call(x))
    }

    pub fn ac_fn(&self) -> Box<Fn(&[FannType]) -> Vec<FannType>> {
        let Ac = self.Ac.clone();
        Box::new(move |x| Ac.borrow().call(x))
    }

    pub fn print(&self) {
        println!("Cacla state: {}", json::encode(&self.state).unwrap());
        println!("CONNECTIONS [V]:  ------------------------------");
        self.V.borrow().print();
        println!("CONNECTIONS [Ac]: ------------------------------");
        self.Ac.borrow().print();
    }
}
