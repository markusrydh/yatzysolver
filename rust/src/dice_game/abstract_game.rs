use rand::Rng;

pub type OutcomeIndex = usize;

pub trait Outcome : std::fmt::Display + std::marker::Sync {
    fn index(&self) -> OutcomeIndex;
    fn initial_probability(&self) -> f32;
} 

pub type MoveIndex = usize;

pub trait Move : std::fmt::Display + std::marker::Sync {
    fn index(&self) -> MoveIndex;
}

pub type SlotMask = u32;
pub type SlotIndex = usize;

pub trait Slot : std::fmt::Display + std::marker::Sync {
    fn slot_mask(&self) -> SlotMask;
    fn index(&self) -> SlotIndex;
    fn bonus(&self) -> bool;
}


pub trait Game<O: Outcome, M: Move, S: Slot> : std::fmt::Display + std::marker::Sync {
    /// Vector with possible outcomes
    fn outcomes<'a>(&'a self) -> &'a Vec<O>;

    /// Vector with possible moves
    fn moves<'a>(&'a self) -> &'a Vec<M>;

    /// Vector with all slots
    fn slots<'a>(&'a self) -> &'a Vec<S>;

    /// Number of moves made before making a slot selection
    fn moves_per_slot(&self) -> u8;

    /// Score in slot for outcome
    fn score(&self, slot: &S, outcome: &O) -> f32;

    // Given a source outcome and a move, list probabilities for each possible outcomes after using move
    fn move_probabilities<'a>(&'a self, from: OutcomeIndex, mov: MoveIndex) -> &'a Vec<(OutcomeIndex, f32)>;

    // Give a random initial outcome with a correct probability distribution
    fn random_initial_outcome<'a>(&'a self) -> &'a O;

    fn bonus_score(&self) -> f32;

    fn bonus_threshold(&self) -> f32;
}


pub fn random_index_from_probabilities<'a, I>(probabilities: I) -> usize where I: Iterator<Item = &'a f32>, {
    let mut rng = rand::thread_rng();
    let mut r = rng.gen::<f32>();
//    println!("Rolled {}", r);
    let mut last_idx: usize = 0;
    for (idx, p) in probabilities.enumerate() {
        if r < *p {
//            println!("p={}, selecting index {}", *p, idx);
            return idx;
        } else {
//            println!("Skipping idx {} with p={}", idx, *p);
            r = r - *p;
            last_idx = idx;
        }
    }
    if r < 0.01 {
        // Rounding issues, the last one should be the one
        return last_idx;
    }
    panic!("Probabilities did not sum to 1.0");
}
