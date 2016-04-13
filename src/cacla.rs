use fann::{Fann, FannType, ActivationFunc, TrainAlgorithm, IncrementalParams};
use rand::distributions::{Normal, IndependentSample};
use std::rc::Rc;
use std::cell::RefCell;
use rand;

#[derive(Clone)]
pub struct Range {
    from: FannType,
    to: FannType,
}

impl Range {
    pub fn new(from: FannType, to: FannType) -> Range {
        Range {
            from: from,
            to: to
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
    fn new(ranges: &Vec<Range>, hidden: u32, output: u32) -> Approx {
        //let mut rs = Vec::with_capacity();
        //rs.clone_from_slice(ranges);
        let rs = ranges.to_vec();
        let mut net = Fann::new(&[ranges.len() as u32, hidden, output]).unwrap();
        //TODO:adjust network params
        net.set_activation_func_hidden(ActivationFunc::SigmoidSymmetric);
        net.set_activation_func_output(ActivationFunc::Linear);
        let train_params = IncrementalParams{learning_momentum: 0.0,
                                             learning_rate: 0.01};
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
    
    fn save(&self, filename: &str) {
        self.net.save(filename);
    }

    fn load(&mut self, filename: &str) {
        // TODO: save and load other settings
        self.net = Fann::from_file(filename).unwrap()
    }
}

pub struct Cacla {
    V: Approx,
    Ac: Approx,
    dim_states: u32,
    dim_actions: u32,
    action: Rc<RefCell<Vec<f64>>>,
    
    alpha: f64,
    beta: f64,
    gamma: f64,
    sigma: f64,
    var: f64,
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
            alpha: alpha,
            beta: beta,
            gamma: gamma,
            sigma: sigma,
            var: 1.0,
            dim_states: state_ranges.len() as u32,
            dim_actions: dim_actions,
            action: Rc::new(RefCell::new(action)),
            V: Approx::new(state_ranges, hidden, 1),
            Ac: Approx::new(state_ranges, hidden, dim_actions),
        }
    }

    pub fn get_action(&self, state: &[FannType]) -> Rc<RefCell<Vec<f64>>> {
        let mu = self.Ac.call(state);
        let mut rng = rand::thread_rng();
        let mut action = self.action.borrow_mut();
        for i in 0..mu.len() {
            let normal = Normal::new(mu[i], self.sigma);
            action[i] = normal.ind_sample(&mut rng);
        }
        self.action.clone()
    }

    pub fn step(&mut self, old_state: &[FannType], new_state: &[FannType],
            action: &[FannType], reward: f64) {
        let old_state_v = self.V.call(old_state);
        let new_state_v = self.V.call(new_state);
        let target = &[reward + self.gamma * new_state_v[0]];
        let td_error = target[0] - old_state_v[0];
        self.V.update(target, old_state);
        if td_error > 0.0 {
            self.var = (1.0 - self.beta) * self.var + self.beta * td_error * td_error;
            let n = (td_error / self.var.sqrt()).ceil() as usize;
            // TODO: print n if n > 5
            if n > 5 {
                println!("n = {}", n);
            }
            for _ in 0..n {
                self.Ac.update(action, old_state)
            }
        }
    }

    pub fn save(&self, path: &str) {
        self.V.save(&(path.to_string() + "/V.net"));
        self.Ac.save(&(path.to_string() + "/Ac.net"));
    }

    pub fn load(&mut self, path: &str) {
        // TODO: save and load other settings
        self.V.load(&(path.to_string() + "/V.net"));
        self.Ac.load(&(path.to_string() + "/Ac.net"));
    }
}
