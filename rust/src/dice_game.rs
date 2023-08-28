pub mod abstract_game;
pub mod game_solver;

use std::collections::HashMap;
use std::fmt::Formatter;
use std::fmt::Error;
use self::abstract_game::*;

#[derive(Debug, Clone)]
pub struct DiceReroll {
    index: MoveIndex,
    rerolled: Vec<bool>
}

impl DiceReroll {
    fn generate_rerolls(num_dice: u8) -> Vec<DiceReroll> {
        let base: u32 = 2;
        let num_rerolls = base.pow(num_dice as u32);
        let q: Vec<u8> = (0..num_dice).collect();
        let mut x = Vec::with_capacity(num_rerolls as usize);
        for r in 0..num_rerolls {
            x.push(DiceReroll { index: r as MoveIndex, rerolled: q.iter().map(|&b| (r & (1<<b)) != 0).collect() })
        }
        return x;
    }

    fn probability(&self, from: &UnorderedDiceOutcome, to: &UnorderedDiceOutcome) -> f32 {
        let n = self.rerolled.len();
        assert!(from.dice.len() == n);
        assert!(to.dice.len() == n);
        assert!(from.num_sides == to.num_sides);
        // Find out what we need to roll by removing the dice kept from the "to" outcome. All kept dice
        // must be found in the "to" outcome, otherwise probability is zero with this re-roll.
        let mut to_roll = to.dice.clone();
        for r in 0..n {
            if !self.rerolled[r] {
                if let Some(pos) = to_roll.iter().position(|x| *x == from.dice[r]) {
                    to_roll.remove(pos);
                } else {
                    return 0.0;
                }
            }
        }
        // Figure out the possibility to roll the dice in to_roll
        // Initially, start by calculating the probability when every dice has to roll exactly the right number.
        // Then we count how many different ways we can reorder the (remaining) dice per every rerolled number
        let mut remaining = to_roll.len() as u8;
        let mut p = (1.0 / (from.num_sides as f32)).powi(remaining as i32);
        for i in 1..=from.num_sides {
            // Count number of dice to roll of value i
            let mut k = 0;
            for r in to_roll.iter() {
                if *r == i {
                    k = k + 1;
                }
            }
            if k > 0 {
                p = p * (Self::choose(remaining,k) as f32);
                remaining -= k;
            }
        }
        return p;
    }

    fn choose(n: u8, k: u8) -> u32 {
        if n == 0 || k == 0 || k == n {
            return 1;
        }
        return Self::choose(n-1, k-1) + Self::choose(n-1, k)
    }
}

impl std::fmt::Display for DiceReroll {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({})", self.rerolled.iter().map(|&x| if x { "x" } else { "-" })
            .collect::<Vec<&str>>()
            .join(","))
    }
}

impl Move for DiceReroll {
    fn index(&self) -> MoveIndex { self.index }
}



#[derive(Debug)]
pub struct UnorderedDiceOutcome {
    index: OutcomeIndex,
    dice: Vec<u8>,
    num_sides: u8,
    initial_probability: f32
}

impl UnorderedDiceOutcome {
    fn generate_outcomes_recurse(num_dices: u8, num_sides: u8, start_at: u8) -> Vec<UnorderedDiceOutcome> {
        let mut outcomes = Vec::new();
        let mut index: OutcomeIndex = 0;
        for d in start_at..=num_sides {
            if num_dices > 1 {
                for o in &UnorderedDiceOutcome::generate_outcomes_recurse(num_dices - 1, num_sides, d) {
                    let mut x = vec![d];
                    for y in &o.dice {
                        x.push(*y);
                    }
                    let initial_probability = o.initial_probability * (1.0/num_sides as f32) * (num_dices as f32) / (o.count_dice_of_value(d) + 1) as f32;
                    outcomes.push(UnorderedDiceOutcome { index, dice: x, num_sides, initial_probability });
                    index += 1;
                }
            } else {
                outcomes.push(UnorderedDiceOutcome { index, dice: vec![d], num_sides, initial_probability: 1.0/num_sides as f32 });
                index += 1;
            }
        }
        return outcomes;
    }

    fn generate_outcomes(num_dices: u8, num_sides: u8) -> Vec<UnorderedDiceOutcome> {
        return UnorderedDiceOutcome::generate_outcomes_recurse(num_dices, num_sides, 1);
    }

    fn count_dice_of_value(&self, value: u8) -> u8 {
        let mut count:u8 = 0;
        for d in self.dice.iter() {
            if *d == value {
                count += 1;
            }
        }
        return count;
    }
}

impl std::fmt::Display for UnorderedDiceOutcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({})", self.dice.iter().map(|&x| x.to_string())
            .collect::<Vec<String>>()
            .join(","))
    }
}

impl Outcome for UnorderedDiceOutcome {
    fn index(&self) -> OutcomeIndex { self.index }
    fn initial_probability(&self) -> f32 { self.initial_probability }
}


pub struct DiceSlotDescription {
    name: &'static str,
    score_lambda: fn(&UnorderedDiceOutcome) -> f32
}

impl DiceSlotDescription {
    fn new(name: &'static str, score_lambda: fn(&UnorderedDiceOutcome) -> f32) -> DiceSlotDescription {
        DiceSlotDescription { name, score_lambda }
    }

    pub fn ones() -> DiceSlotDescription {
        DiceSlotDescription::new("Ones", |o| o.count_dice_of_value(1) as f32 * 1.0)
    }

    pub fn twos() -> DiceSlotDescription {
        DiceSlotDescription::new("Twos", |o| o.count_dice_of_value(2) as f32 * 2.0)
    }

    pub fn threes() -> DiceSlotDescription {
        DiceSlotDescription::new("Threes", |o| o.count_dice_of_value(3) as f32 * 3.0)
    }

    pub fn fours() -> DiceSlotDescription {
        DiceSlotDescription::new("Fours", |o| o.count_dice_of_value(4) as f32 * 4.0)
    }

    pub fn fives() -> DiceSlotDescription {
        DiceSlotDescription::new("Fives", |o| o.count_dice_of_value(5) as f32 * 5.0)
    }

    pub fn sixes() -> DiceSlotDescription {
        DiceSlotDescription::new("Sixes", |o| o.count_dice_of_value(6) as f32 * 6.0)
    }

    pub fn one_pair() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            for x in (1..=o.num_sides).rev() {
                if o.count_dice_of_value(x) >= 2 {
                    return 2.0*(x as f32);
                }
            }
            0.0
        }
        DiceSlotDescription::new("One pair", score)
    }

    pub fn two_pairs() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            let mut score: f32 = 0.0;
            for x in (1..=o.num_sides).rev() {
                if o.count_dice_of_value(x) >= 2 {
                    if score > 0.0 {
                        return score + 2.0*(x as f32);
                    } else {
                        score = 2.0*(x as f32);
                    }
                }
            }
            0.0
        }
        DiceSlotDescription::new("Two pairs", score)
    }

    pub fn three_of_a_kind() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            for x in (1..=o.num_sides).rev() {
                if o.count_dice_of_value(x) >= 3 {
                    return 3.0*(x as f32);
                }
            }
            0.0
        }
        DiceSlotDescription::new("Three of a kind", score)
    }

    pub fn four_of_a_kind() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            for x in (1..=o.num_sides).rev() {
                if o.count_dice_of_value(x) >= 4 {
                    return 4.0*(x as f32);
                }
            }
            0.0
        }
        DiceSlotDescription::new("Four of a kind", score)
    }

    pub fn yatzy() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            for x in 1..=o.num_sides {
                if o.count_dice_of_value(x) as usize == o.dice.len() {
                    return 50.0;
                }
            }
            0.0
        }
        DiceSlotDescription::new("YATZY", score)
    }

    pub fn chance() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            let mut score = 0.0;
            for x in 1..=o.num_sides {
                score = score + (o.count_dice_of_value(x) * x) as f32;
            }
            return score;
        }
        DiceSlotDescription::new("Chance", score)
    }

    pub fn small_straight() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            let mut score = 0.0;
            for x in 1..=o.dice.len() {
                if o.count_dice_of_value(x as u8) != 1 {
                    return 0.0;
                }
                score = score + x as f32;
            }
            return score;
        }
        DiceSlotDescription::new("Small straight", score)
    }

    pub fn large_straight() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            let mut score = 0.0;
            for x in (o.num_sides - o.dice.len() as u8 + 1)..=o.num_sides {
                if o.count_dice_of_value(x as u8) != 1 {
                    return 0.0;
                }
                score = score + x as f32;
            }
            return score;
        }
        DiceSlotDescription::new("Large straight", score)
    }

    pub fn full_house() -> DiceSlotDescription {
        fn score(o: &UnorderedDiceOutcome) -> f32 {
            if o.dice.len() % 2 == 0 {
                return 0.0;
            }
            // n is the bigger number of dice with a certain number of pips
            let n = ((o.dice.len() + 1) / 2) as u8;
            let mut n_pips: u8 = 0;
            for x in 1..=o.num_sides {
                if o.count_dice_of_value(x as u8) == n {
                    n_pips = x;                    
                }
            }
            if n_pips == 0 {
                return 0.0;
            }
            for x in 1..=o.num_sides {
                if x != n_pips && o.count_dice_of_value(x as u8) == n - 1 {
                    return (n_pips * n + x * (n-1)) as f32;
                }
            }
            return 0.0;
        }
        DiceSlotDescription::new("Full house", score)
    }
}

pub struct DiceSlot {
    index: SlotIndex,
    name: &'static str,
    slot_mask: SlotMask,
    score_lambda: fn(&UnorderedDiceOutcome) -> f32
}

impl DiceSlot {
    fn new(dice_slot_description: &DiceSlotDescription, index: SlotIndex) -> DiceSlot {
        DiceSlot {
            name: dice_slot_description.name,
            index,
            slot_mask: (1<<index),
            score_lambda: dice_slot_description.score_lambda
        }
    }
    fn score(&self, outcome: &UnorderedDiceOutcome) -> f32 {
        (self.score_lambda)(outcome)
    }
}

impl Slot for DiceSlot {
    fn index(&self) -> SlotIndex { self.index }
    fn slot_mask(&self) -> SlotMask { self.slot_mask }
}

impl std::fmt::Display for DiceSlot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.name)
    }
}

pub struct DiceGame {
    outcomes: Vec<UnorderedDiceOutcome>,
    moves: Vec<DiceReroll>,
    slots: Vec<DiceSlot>,
    num_dice: u8,
    num_sides: u8,
    num_rolls: u8,
    // Scores as [slot-index][outcome-index]
    scores: Vec<Vec<f32>>,
    // Given starting outcome and a move, return all possible outcomes with probabilities
    probabilities: HashMap<(OutcomeIndex, MoveIndex), Vec<(OutcomeIndex, f32)>>
}

impl DiceGame {
    pub fn new(num_dice: u8, num_sides: u8, num_rolls: u8, slot_descriptions: Vec<DiceSlotDescription>) -> DiceGame {
        let outcomes = UnorderedDiceOutcome::generate_outcomes(num_dice, num_sides);
        let moves = DiceReroll::generate_rerolls(num_dice);

        let mut slots = Vec::<DiceSlot>::with_capacity(slot_descriptions.len());
        for (idx, slot_description) in slot_descriptions.iter().enumerate() {
            slots.push(DiceSlot::new(slot_description, idx));
        }

        let mut all_scores = Vec::<Vec<f32>>::with_capacity(slots.len());
        for slot in slots.iter() {
            let slot_scores = outcomes.iter().map(|o| slot.score(o)).collect();
            all_scores.push(slot_scores);
        }

        let mut probabilities: HashMap<(OutcomeIndex, MoveIndex), Vec<(OutcomeIndex, f32)>> = HashMap::new();
        for from in outcomes.iter() {
            for m in moves.iter() {
                let p = outcomes.iter()
                    .map(|to| (to.index, m.probability(&from, &to)))
                    .filter(|v| v.1 > 0.0)
                    .collect();
                probabilities.insert((from.index, m.index), p);
            }
        }
        DiceGame { outcomes, moves, slots, num_dice, num_sides, num_rolls, scores: all_scores, probabilities }
    }

    pub fn find_outcome(&self, dice: &Vec<u8>) -> Option<&UnorderedDiceOutcome> {
        let mut sorted_dice : Vec<u8> = dice.clone();
        sorted_dice.sort();
        return self.outcomes.iter().find(|o| o.dice == sorted_dice);
    }

    pub fn find_move(&self, rerolled: &Vec<bool>) -> Option<&DiceReroll> {
        // let mut sorted_dice : Vec<u8> = dice.clone();
        // sorted_dice.sort();
        return self.moves.iter().find(|m| m.rerolled == *rerolled);
    }
}

impl Game<UnorderedDiceOutcome, DiceReroll, DiceSlot> for DiceGame {
    fn outcomes<'a>(&'a self) -> &'a Vec<UnorderedDiceOutcome> { &self.outcomes }
    fn moves<'a>(&'a self) -> &'a Vec<DiceReroll> { &self.moves }
    fn slots<'a>(&'a self) -> &'a Vec<DiceSlot> { &self.slots }
    fn moves_per_slot(&self) -> u8 { self.num_rolls }

    fn score(&self, slot: &DiceSlot, outcome: &UnorderedDiceOutcome) -> f32 {
        self.scores[slot.index][outcome.index]

    }

    fn move_probabilities<'a>(&'a self, from: OutcomeIndex, mov: MoveIndex) -> &'a Vec<(OutcomeIndex, f32)> {
        &self.probabilities[&(from, mov)]
    }

    fn random_initial_outcome<'a>(&'a self) -> &'a UnorderedDiceOutcome {
        let idx = random_index_from_probabilities(self.outcomes.iter().map(|o| &o.initial_probability));
        return &self.outcomes[idx];
    }
}


impl std::fmt::Display for DiceGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "#dice: {}, #sides/dice: {}, #rolls/round: {}", self.num_dice, self.num_sides, self.num_rolls)
    }
}
