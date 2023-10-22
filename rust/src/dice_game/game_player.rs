use std::fmt::Error;
use std::fmt::Formatter;

use crate::game_solver::*;
use crate::abstract_game::*;

#[derive(Debug)]
pub struct GameProtocol{
    total_slot_score : f32,
    slot_scores : Vec<f32>,
    bonus : bool,
    total_game_score : f32
}

pub struct SlotStatistics {
    sample_count: f32,
    score_sum: f32, 
    square_sum: f32,
    zero_count: u32,
    histogram: Histogram
}

pub struct GameStatistics {
    slot_names: Vec<String>,
    total_slot_score: SlotStatistics,
    total_game_score: SlotStatistics,
    bonus: SlotStatistics,
    all_slots_statistics: Vec<SlotStatistics>
}

pub struct Histogram {
    // Number of data points for different scores rounded to closest integer.
    // The rounded score is the index into the vector
    data: Vec<u32>,
}

impl Histogram {
    fn new() -> Histogram {
        Histogram { data: Vec::new() }
    }

    fn add_sample(&mut self, score: f32) {
        let int_score = score as usize;
        // Dynamically increase size of data vector according to needs. Works if scores are not terribly large.
        while int_score >= self.data.len() {
            // Ugly...
            self.data.push(0);
        }
        self.data[int_score] += 1;
    }
}

impl std::fmt::Display for Histogram {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut res: Result<(), Error> = Ok(());
        for i in 0..self.data.len() {
            let count = self.data[i];
//            if count > 0 {
                res = res.and_then(|_x| writeln!(f, "{}\t{}", i, count));
//            }
        }
        return res;
    }
}


impl SlotStatistics{
    fn new() -> SlotStatistics {
        SlotStatistics { sample_count: 0.0, score_sum: 0.0, square_sum: 0.0, zero_count: 0, histogram: Histogram::new() }
    }

    pub fn add_sample(&mut self, score: f32) {
        self.sample_count += 1.0;
        self.score_sum += score;
        self.square_sum += score.powi(2);
        if score == 0.0 {
            self.zero_count += 1;
        }
        self.histogram.add_sample(score)
    }
    pub fn average(&self) -> f32 {
        return self.score_sum / self.sample_count;
    }
    pub fn std_dev(&self) -> f32 {
        let average = self.average();
        return ((self.square_sum/self.sample_count)-average.powi(2)).sqrt();
    }    
}

impl std::fmt::Display for SlotStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "#avg: {}, #std dev: {}, #zeroes: {}", self.average(), self.std_dev(), self.zero_count)
    }
}

impl GameStatistics {
    fn new<G: Game<O, M, S>, O: Outcome, M: Move, S: Slot>(game: &G) -> GameStatistics {
        let mut all_slot_statistics = Vec::<SlotStatistics>::new();
        let mut slot_names = Vec::<String>::new();
        for s in game.slots() {
            all_slot_statistics.push(SlotStatistics::new());
            slot_names.push(s.to_string());
        }
        GameStatistics {
            slot_names,
            total_slot_score: SlotStatistics::new(),
            total_game_score: SlotStatistics::new(),
            bonus: SlotStatistics::new(),
            all_slots_statistics: all_slot_statistics
        }
    }

    pub fn add_protocol(&mut self, protocol: GameProtocol) {
        self.total_game_score.add_sample(protocol.total_game_score);
        self.total_slot_score.add_sample(protocol.total_slot_score);
        self.bonus.add_sample(if protocol.bonus { 1.0 } else { 0.0 });
        for i in 0..protocol.slot_scores.len() {
            self.all_slots_statistics[i].add_sample(protocol.slot_scores[i]);
        }
    }

    pub fn print_histogram(&self) {
        println!("Total game score histogram:");
        print!("{}", self.total_game_score.histogram);
        println!("\nTotal slot score histogram:");
        print!("{}", self.total_slot_score.histogram);
        for i in 0..self.slot_names.len() {
            println!("\n{} histogram:", self.slot_names[i]);
            print!("{}", self.all_slots_statistics[i].histogram);
        }
    }
}    

impl std::fmt::Display for GameStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut res: Result<(), Error> = Ok(());
        res = res.and_then(|_x: ()| write!(f, "Total game score: {}\n", self.total_game_score));
        res = res.and_then(|_x| write!(f, "Total slot score: {}\n", self.total_slot_score));
        res = res.and_then(|_x| write!(f, "Bonus probability: {}\n", self.bonus));
        for i in 0..self.slot_names.len() {
            res = res.and_then(|_x| write!(f, "  {}: {}\n", self.slot_names[i], self.all_slots_statistics[i]));
        }
        return res;
    }
}

pub fn play_games<G: Game<O, M, S>, O: Outcome, M: Move, S: Slot>(solver:&GameSolver<G, O, M, S>, game_count: u32) -> GameStatistics {
    let mut statistics = GameStatistics::new(solver.game);
    for _i in 0..game_count {
        let protocol = play_whole_game(solver);
        statistics.add_protocol(protocol);
    }
    return statistics;
}

pub fn play_whole_game<G: Game<O, M, S>, O: Outcome, M: Move, S: Slot>(solver:&GameSolver<G, O, M, S>) -> GameProtocol{
    let start_pos = solver.initial_position(None);
    let mut protocol = play_game(&start_pos,  GameProtocol{
        total_slot_score : 0.0, 
        slot_scores: vec![0.0;start_pos.available_slots().len()], 
        bonus: false,
        total_game_score: 0.0
    });
    protocol.total_game_score = protocol.total_slot_score;

    let all_slots = solver.game.slots();
    let mut bonus_sum : f32 = 0.0;
    for slot in all_slots{
        if slot.bonus(){
            bonus_sum += protocol.slot_scores[slot.index()];
        }
    }
    if bonus_sum >= solver.game.bonus_threshold(){
        protocol.bonus = true;
        protocol.total_game_score += solver.game.bonus_score();
    }
    return protocol;
}

fn play_game<'a,G: Game<O, M, S>, O: Outcome, M: Move, S: Slot>(position: &Position<'a, G, O, M, S>, mut protocol: GameProtocol) -> GameProtocol {
//    println!("Current position\n================\n{}", position);

    if let Some(filled_slot) = position.best_slot_to_fill(){
        protocol.total_slot_score += filled_slot.score;
        protocol.slot_scores[filled_slot.index] = filled_slot.score;
    }

    if let Some(next_position) = position.follow_best_move() {
        return play_game(&next_position, protocol);
    } else {
        return protocol;
    }
}