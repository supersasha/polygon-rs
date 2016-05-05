//#![feature(custom_derive, plugin)]
use fann::{Fann, FannType, ActivationFunc, TrainAlgorithm, IncrementalParams};
use rand::distributions::{Normal, IndependentSample};
use std::rc::Rc;
use std::cell::RefCell;
use rand;
use rustc_serialize::json;
use std::fs::File;
use std::io::prelude::*;
use std::path;

#[derive(Clone)]
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
struct CaclaState {
    action: Rc<RefCell<Vec<f64>>>,    
    alpha: f64,
    beta: f64,
    gamma: f64,
    sigma: f64,
    var: f64,
}

pub struct Cacla {
    V: Approx,
    Ac: Approx,
    state: CaclaState,
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
                sigma: sigma,
                var: 1.0,
                action: Rc::new(RefCell::new(action))
            },
            V: Approx::new(state_ranges, hidden, 1, alpha),
            Ac: Approx::new(state_ranges, hidden, dim_actions, alpha)
        }
    }

    pub fn get_action(&self, state: &[FannType], wander_more: bool) -> Rc<RefCell<Vec<f64>>> {
        let mu = self.Ac.call(state);
        let mut rng = rand::thread_rng();
        let mut action = self.state.action.borrow_mut();
        let mut sigma = self.state.sigma;
        if wander_more {
            sigma = 1.0
        } 
        for i in 0..mu.len() {
            let normal = Normal::new(mu[i], sigma);
            action[i] = normal.ind_sample(&mut rng);
        }
        self.state.action.clone()
    }

    pub fn step(&mut self, old_state: &[FannType], new_state: &[FannType],
            action: &[FannType], reward: f64) {
        let old_state_v = self.V.call(old_state);
        let new_state_v = self.V.call(new_state);
        let target = &[reward + self.state.gamma * new_state_v[0]];
        let td_error = target[0] - old_state_v[0];
        self.V.update(target, old_state);
        if td_error > 0.0 {
            self.state.var = (1.0 - self.state.beta) * self.state.var
                            + self.state.beta * td_error * td_error;
            let n = (td_error / self.state.var.sqrt()).ceil() as usize;
            // TODO: print n if n > 5
            if n > 5 {
                println!("n = {}", n);
            }
            for _ in 0..n {
                self.Ac.update(action, old_state)
            }
        }
    }
    
    pub fn save(&self, dir: &path::PathBuf) {
        let mut f = File::create(&dir.join("/cacla.state")).unwrap();
        write!(f, "{}", json::encode(&self.state).unwrap());
        self.V.save(&dir.join("/V.net"));
        self.Ac.save(&dir.join("/Ac.net"));
    }

    pub fn load(&mut self, dir: &path::PathBuf) {
        let mut f = File::open(&dir.join("/cacla.state")).unwrap();
        let mut js = String::new();
        f.read_to_string(&mut js);
        self.state = json::decode(&js).unwrap();
        self.V.load(&dir.join("/V.net"));
        self.Ac.load(&dir.join("/Ac.net"));
    }
    
    pub fn print(&self) {
        println!("Cacla state: {}", json::encode(&self.state).unwrap());
        println!("CONNECTIONS [V]:  ------------------------------");
        self.V.print();
        println!("CONNECTIONS [Ac]: ------------------------------");
        self.Ac.print();
    }
}
