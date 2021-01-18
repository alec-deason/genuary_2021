use std::collections::{HashMap, HashSet};
use nannou::rand::{rngs::SmallRng, Rng, SeedableRng, prelude::*};
use mathru::statistics::test::{ChiSquare, Test};

#[derive(Clone, Debug)]
struct Rule {
    neighborhood: Neighborhood,
    current: State,
    state_of_interest: State,
    target: u32,
    comparison: Comparison,
    result: State,
}

impl Rule {
    fn new_random<R: Rng>(state_count: u32, rng: &mut R) -> Self {
        let neighborhood = Neighborhood::new_random(rng);
        Self {
            neighborhood,
            current: State(rng.gen_range(0, state_count)),
            state_of_interest: State(rng.gen_range(0, state_count)),
            target: rng.gen_range(0, neighborhood.max_count()),
            comparison: Comparison::new_random(neighborhood.max_count(), rng),
            result: State(rng.gen_range(0, state_count)),
        }
    }

    fn mutate<R: Rng>(&mut self, state_count: u32, rng: &mut R) {
        let v = rng.gen::<f32>();
        if v > 0.99 {
            self.neighborhood.mutate(rng);
        } else if v > 0.9 {
            self.current = State(rng.gen_range(0, state_count));
        } else if v > 0.8 {
            self.state_of_interest = State(rng.gen_range(0, state_count));
        } else if v > 0.7 {
            self.comparison.mutate(self.neighborhood.max_count(), rng);
        } else if v > 0.5 {
            self.target = (self.target as i32 + rng.gen_range(-1, 1)).max(0).min(self.neighborhood.max_count() as i32) as u32;
        } else {
            self.result = State(rng.gen_range(0, state_count));
        }
        *self = Rule::new_random(state_count, rng);
    }

    fn apply(&self, i: u32, size: (u32, u32), world: &[State]) -> Option<State> {
        if world[i as usize] != self.current {
            return None
        }

        let mut count = 0;
        for j in self.neighborhood.neighbors(i, size) {
            if world[j as usize] == self.state_of_interest {
                count += 1;
            }
        }
        if self.comparison.cmp(count, self.target) {
            Some(self.result)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Neighborhood {
    VonNeuman,
    Conway,
    ManhattanDistance(u32),
}

impl Neighborhood {
    fn new_random<R: Rng>(rng: &mut R) -> Self {
        let distance = rng.gen_range(1, 4);
        //*[Neighborhood::VonNeuman, Neighborhood::Conway, Neighborhood::ManhattanDistance(distance)].choose(rng).unwrap()
        Neighborhood::ManhattanDistance(distance)
    }

    fn mutate<R: Rng>(&mut self, rng: &mut R) {
        *self = Neighborhood::new_random(rng);
    }

    fn max_count(&self) -> u32 {
        match self {
            Neighborhood::VonNeuman => 4,
            Neighborhood::Conway => 8,
            Neighborhood::ManhattanDistance(r) => (2 * r).pow(2) -1,
        }
    }

    fn neighbors(&self, i: u32, size: (u32, u32)) -> Vec<u32> {
        let x = i % size.0;
        let y = i / size.0;
        match self {
            Neighborhood::VonNeuman => {
                [(1, 0), (0, 1), (-1, 0), (0, -1)].iter().filter_map(|(dx, dy)| {
                    let x = x as i32 + dx;
                    let y = y as i32 + dy;
                    if x >= 0 && x < size.0 as i32 && y >= 0 && y < size.1 as i32 {
                        Some(x as u32 + y as u32 * size.0)
                    } else {
                        None
                    }
                }).collect()
            },
            Neighborhood::Conway => {
                [(1, 0), (1, 1), (1, -1), (0, 1), (-1, 0), (-1, 1), (-1, -1), (0, -1)].iter().filter_map(|(dx, dy)| {
                    let x = x as i32 + dx;
                    let y = y as i32 + dy;
                    if x >= 0 && x < size.0 as i32 && y >= 0 && y < size.1 as i32 {
                        Some(x as u32 + y as u32 * size.0)
                    } else {
                        None
                    }
                }).collect()
            },
            Neighborhood::ManhattanDistance(r) => {
                let mut result = Vec::with_capacity(self.max_count() as usize);
                for dx in -(*r as i32)..*r as i32 {
                    for dy in -(*r as i32)..*r as i32 {
                        let x = x as i32 + dx;
                        let y = y as i32 + dy;
                        if x >= 0 && x < size.0 as i32 && y >= 0 && y < size.1 as i32 {
                            result.push(x as u32 + y as u32 * size.0)
                        }
                    }
                }
                result
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct State(u32);

#[derive(Copy, Clone, Debug)]
enum Comparison {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    AbsDiff(u32),
}

impl Comparison {
    fn new_random<R: Rng>(max_count: u32, rng: &mut R) -> Self {
        let d = rng.gen_range(1, max_count);
        *[Comparison::Equal, Comparison::GreaterThan, Comparison::GreaterThanOrEqual, Comparison::LessThan, Comparison::LessThanOrEqual, Comparison::AbsDiff(d)].choose(rng).unwrap()
    }

    fn mutate<R: Rng>(&mut self, max_count: u32, rng: &mut R) {
        *self = Comparison::new_random(max_count, rng);
    }

    fn cmp(&self, a: u32, b: u32) -> bool {
        match self {
            Comparison::Equal => {
                a == b
            },
            Comparison::GreaterThan => {
                a > b
            },
            Comparison::GreaterThanOrEqual => {
                a >= b
            },
            Comparison::LessThan => {
                a < b
            },
            Comparison::LessThanOrEqual => {
                a <= b
            },
            Comparison::AbsDiff(d) => {
                (a as i32 -b as i32).abs() as u32 == *d
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    rules: Vec<Rule>,
    state: Vec<State>,
    initial_state: Vec<State>,
    time_in_state: Vec<u32>,
    size: (u32, u32),
    state_count: u32,
    transition_times: Vec<u32>,
    proportion_changed: Vec<f32>,
    frames: Vec<Vec<State>>,
}

impl Model {
    pub fn new_random<R: Rng>(width: u32, height: u32, rule_count: u32, state_count: u32, rng: &mut R) -> Self {
        let initial_state:Vec<State> = (0..width*height).map(|_| State(rng.gen_range(0, state_count))).collect();
        Self {
            rules: (0..rule_count).map(|_| Rule::new_random(state_count, rng)).collect(),
            state: initial_state.clone(),
            initial_state,
            time_in_state: vec![0; (width*height) as usize],
            size: (width, height),
            state_count,
            transition_times: vec![],
            proportion_changed: vec![],
            frames: vec![],
        }
    }

    pub fn mutate<R: Rng>(&mut self, rng: &mut R) {
        let rule = self.rules.iter_mut().choose(rng).unwrap();
        rule.mutate(self.state_count, rng);
    }

    pub fn step_for(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    pub fn reset(&mut self) {
        self.state = self.initial_state.clone();
        self.transition_times.clear();
        self.time_in_state = vec![0; (self.size.0*self.size.1) as usize];
    }

    pub fn reset_random<R: Rng>(&mut self, rng: &mut R) {
        let initial_state:Vec<State> = (0..self.size.0*self.size.1).map(|_| State(rng.gen_range(0, self.state_count()))).collect();
        self.state = initial_state.clone();
        self.initial_state = initial_state;
        self.transition_times.clear();
        self.time_in_state = vec![0; (self.size.0*self.size.1) as usize];
    }

    pub fn step(&mut self) {
        let mut swap = vec![State(0); self.state.len()];
        let mut number_changed = 0;
        for i in 0..self.state.len() {
            let mut did_apply = false;
            let mut did_change = false;
            for rule in &self.rules {
                if let Some(new_state) = rule.apply(i as u32, self.size, &self.state) {
                    did_apply = true;
                    swap[i as usize] = new_state;
                    if self.state[i as usize] != new_state {
                        did_change = true;
                    }
                    break;
                }
            }
            if !did_apply {
                swap[i as usize] = self.state[i as usize];
            }
            if did_change {
                number_changed += 1;
                self.transition_times.push(self.time_in_state[i as usize]);
                self.time_in_state[i as usize] = 0;
            } else {
                self.time_in_state[i as usize] += 1;
            }
        }
        self.proportion_changed.push(number_changed as f32 / self.state.len() as f32);
        std::mem::swap(&mut self.state, &mut swap);
        self.frames.push(swap);
    }

    pub fn rule_count(&self) -> u32 {
        self.rules.len() as u32
    }
    pub fn state_count(&self) -> u32 {
        self.state_count
    }

    pub fn states(&self) -> Vec<u32> {
        self.state.iter().map(|s| s.0).collect()
    }

    pub fn stats(&self) -> Stats {
        let mut per_state:HashMap<_, _> = (0..self.state_count()).map(|s| (State(s), 0)).collect();
        for s in &self.state {
            *per_state.entry(*s).or_insert(0usize) += 1;
        }

        let observed:Vec<_> = self.transition_times.iter().chain(self.time_in_state.iter()).map(|t| *t as f32).choose_multiple(&mut thread_rng(), 1000);
        let sd = observed.iter().sum::<f32>();
        let time_in_state = sd / observed.len() as f32;


        let mut rng = thread_rng();

        let path_length = 16;
        let mut path_score = 0.0;
        let mut samples = 0;
        for frame in (0..self.frames.len()-path_length).choose_multiple(&mut rng, 100) {
            for i in (0..self.state.len()).choose_multiple(&mut rng, 100) {
                let mut velocity = (0,0);
                samples += 1;
                let mut loc = i as u32;

                let state = self.frames[frame][i];
                for d in 1..path_length {
                    if self.frames[frame+d][i as usize] == state {
                        break
                    }
                    let mut found = false;
                    for j in Neighborhood::Conway.neighbors(loc as u32, self.size) {
                        if self.frames[frame+d][j as usize] == state {
                            found = true;
                            velocity.0 += loc % self.size.0 - j % self.size.0;
                            velocity.1 += loc / self.size.0 - j / self.size.0;
                            loc = j;
                            break
                        }
                    }
                    if !found {
                        break
                    }
                }
                path_score += (velocity.0.pow(2) + velocity.1.pow(2)) as f32 / path_length as f32;
            }
        }
        path_score /= samples as f32;
        Stats {
            time_in_state,
            path_score,
        }
    }
}

pub struct Stats {
    pub time_in_state: f32,
    pub path_score: f32,
}
