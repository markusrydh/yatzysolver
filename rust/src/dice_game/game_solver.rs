use std::marker::PhantomData;
use std::collections::HashMap;
use std::fmt::Formatter;
use rayon::prelude::*;

use crate::abstract_game::*;

pub struct GameSolver<'a, G: Game<O, M, S>, O: Outcome, M: Move, S: Slot> {
    game: &'a G,
    initial_stage: Option<Stage>,
    _o: PhantomData<O>,
    _m: PhantomData<M>,
    _s: PhantomData<S>,
}

impl <'a, G: Game<O, M, S>, O: Outcome, M: Move, S: Slot> GameSolver<'a, G, O, M, S> {
    pub fn new(game: &'a G) -> GameSolver<G, O, M, S> {
        GameSolver { game, initial_stage: None, _o: PhantomData, _m: PhantomData, _s: PhantomData }
    }

    pub fn solve(&mut self) {
        let all_slots_mask = self.game.slots().iter().fold(0, |acc, s| (acc | s.slot_mask()));
        let stage = Stage::from_end(all_slots_mask);
        self.initial_stage = Some(self.solve_stage(stage));
    }

    // Create a new stage which comes before (in game time) the given previous stage
    fn stage_from_previous<'s>(&self, prev_stage: Stage) -> Stage {
        assert!(prev_stage.is_solved);

        println!("Generating new stage based on stage #{}...", prev_stage.number);
        let mut new_rounds = HashMap::new();
        let outcomes = self.game.outcomes();
        let moves_per_slot: usize = self.game.moves_per_slot().into();
        for prev_round in prev_stage.rounds.values() {
            for s in self.game.slots().iter() {
                let slot_mask = s.slot_mask();
                if prev_round.slots & slot_mask > 0 {
                    // It is possible to un-fill slot s, i.e. move is possible
                    let new_mask = prev_round.slots ^ slot_mask;
                    let new_round = &mut new_rounds.entry(new_mask).or_insert(Round::new(new_mask, self.game.moves_per_slot(), outcomes.len()));
                    for o in outcomes.iter() {
                        // Calculate expected score for outcome o
                        let expected_score = self.game.score(s, o) + prev_round.expected_score.unwrap();
                        // let b1: &Vec<BestMoveWithScore> = &new_round.best_moves[moves_per_slot-1];
                        // let mut best = &mut b1[o.index()];
                        let mut best = &mut new_round.best_moves[moves_per_slot-1][o.index()];
                        if best.best_move == BestMove::Unknown || best.expected_score < expected_score {
                            best.expected_score = expected_score;
                            best.best_move = BestMove::SelectSlot(s.index());
                        }
                    }
                }
            }
        }
        if new_rounds.len() > 0 {
            let new_stage_number = prev_stage.number + 1;
            Stage {
                previous_stage: Some(Box::new(prev_stage)),
                number: new_stage_number,
                is_solved: false,
                rounds: new_rounds
            }
        } else {
            prev_stage
        }
    }

    fn solve_stage(&self, mut stage: Stage) -> Stage {
        if !stage.is_solved {
            println!("Solving stage {} with #{} rounds", stage.number, stage.rounds.len());
            stage.rounds.par_iter_mut().for_each(|(j, r)| self.solve_round(r));
            stage.is_solved = true;
        }
        let next_stage = self.stage_from_previous(stage);
        if next_stage.is_solved {
            println!("Finished with optimal expected score {}!", next_stage.rounds[&0 /* empty slotmask */].expected_score.unwrap());
            return next_stage
        } else {
            return self.solve_stage(next_stage);
        }
    }

    fn solve_round(&self, round: &mut Round) {
//        println!("Solving round {:15b}...", round.slots);
        while round.solved_move_number_index > 0 {
//            println!("Solving round {:15b} move {}...", round.slots, round.solved_move_number_index - 1);
            for o in self.game.outcomes().iter() {
//                let mut best = BestMoveWithScore { best_move: BestMove::Unknown, expected_score: 0.0 };
                for m in self.game.moves().iter() {
                    let v = &round.best_moves[round.solved_move_number_index];
                    let m_score = self.game.move_probabilities(o.index(), m.index()).iter().fold(0.0, |new_score, (o_idx, p)| new_score + v[*o_idx].expected_score * *p);
                    let best = round.best_moves[round.solved_move_number_index - 1][o.index()];
                    if best.best_move == BestMove::Unknown || best.expected_score < m_score {
                        round.best_moves[round.solved_move_number_index - 1][o.index()] = BestMoveWithScore {
                            best_move: BestMove::Move(m.index()),
                            expected_score: m_score
                        };
                    }
                }
//                println!("Best move for {}: {:?}", o, round.best_moves[round.solved_move_number_index - 1][o.index()]);
            }
            round.solved_move_number_index -= 1;
        }

        // Find rounds expected score from the scores for the inital outcomes of the round
        // weighed by probabilities of the outcomes.
        let v = &round.best_moves[0];
        let mut round_score = 0.0;
        for o in self.game.outcomes().iter() {
            round_score += o.initial_probability() * v[o.index()].expected_score;
        }
        round.expected_score = Some(round_score);
//        println!("Solved round {:15b} with expected score {}", round.slots, round_score);
    }

    pub fn get_best_move(&self, slot_mask: SlotMask, move_number: usize, outcome_index: OutcomeIndex) -> Option<BestMoveWithScore> {
        if let Some(initial_stage) = &self.initial_stage {
            fn search_round<'s>(stage: &'s Stage, slot_mask: SlotMask) -> Option<&'s Round> {
                if let Some(r) = stage.rounds.get(&slot_mask) {
                    return Some(r);
                }
                if let Some(prev_stage) = &stage.previous_stage {
                    return search_round(&prev_stage, slot_mask);
                }
                None
            }
            if let Some(r) = search_round(&initial_stage, slot_mask) {
                let o_vec: Option<&Vec<BestMoveWithScore>> = r.best_moves.get(move_number);
                return o_vec.map(|o_vec| o_vec.get(outcome_index)).flatten().map(|x| *x);
            }
        }
        None
    }

    pub fn initial_position(&'a self, initial_outcome: Option<&O>) -> Position<'a, G, O, M, S> {
        Position {
            solver: self,
            slot_mask: 0,
            move_number: 0,
            outcome_index: initial_outcome.map_or(self.game.random_initial_outcome().index(), |o| o.index()),
            score: 0.0
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum BestMove {
    SelectSlot(SlotIndex),
    Move(MoveIndex),
    Unknown
}


#[derive(Debug, Copy, Clone)]
struct BestMoveWithScore {
    best_move: BestMove,
    expected_score: f32
}

// A stage consists of all rounds with the same amount of slots filled.
// Stages are constructed from last stage (Round with all slots filled),
// to the second last stage (all rounds with all but one slots filled) and so on
// backwards up to the first stage (single round with no slots filled).
pub struct Stage {
    previous_stage: Option<Box<Stage>>,
    number: u8,
    is_solved: bool,
    rounds: HashMap<SlotMask, Round>
}

impl Stage {
    // Create a new stage, which is the last (in game time) where all slots are filled
    fn from_end(all_slots_mask: SlotMask) -> Stage {
        let mut round = Round::new(all_slots_mask, 0, 0);
        round.expected_score = Some(0.0);
        let map = HashMap::from([(all_slots_mask, round)]);
        Stage { previous_stage: None, number: 1, is_solved: true, rounds: map }
    }
}

struct Round {
    // Bitmask of filled slots at the start of this round
    slots: SlotMask,
    // Expected score at the start of this round when solved
    expected_score: Option<f32>,
    // Index of solved move number, will progress from num_moves_per_slot-1 and finish at 0
    solved_move_number_index: usize,
    // De-reference is made as best_moves[<move-number>][<outcome-number>]
    best_moves: Vec<Vec<BestMoveWithScore>>
}

impl Round {
    fn new(slots: SlotMask, num_moves_per_slot: u8, num_outcomes: usize) -> Round {
        let best_moves = vec![vec![BestMoveWithScore { best_move: BestMove::Unknown, expected_score: 0.0 }; num_outcomes]; num_moves_per_slot.into()];
        let solved_move_number_index = if num_moves_per_slot > 0 { (num_moves_per_slot-1).into() } else { 0 };
        Round { slots, expected_score: None, best_moves, solved_move_number_index }
    }
}




pub struct Position<'a, G: Game<O, M, S>, O: Outcome, M: Move, S: Slot> {
    solver: &'a GameSolver<'a, G, O, M, S>,
    slot_mask: SlotMask,
    move_number: usize,
    outcome_index: OutcomeIndex,
    score: f32
}

impl <'a, G: Game<O, M, S>, O: Outcome, M: Move, S: Slot> Position<'a, G, O, M, S> {
    pub fn is_final(&self) -> bool {
        self.follow_best_move().is_some()
    }
    pub fn available_slots(&self) -> Vec<&'a S> {
        self.solver.game.slots().iter().filter(|s| (s.slot_mask() & self.slot_mask) == 0).collect()
    }
    pub fn filled_slots(&self) -> Vec<&'a S> {
        self.solver.game.slots().iter().filter(|s| (s.slot_mask() & self.slot_mask) != 0).collect()
    }
    pub fn follow_best_move(&'a self) -> Option<Position<'a, G, O, M, S>> {
        if let Some(best_move_with_score) = self.get_best_move() {
            return match best_move_with_score.best_move {
                BestMove::SelectSlot(slot_index) => {
                    let slot = &self.solver.game.slots()[slot_index];
                    let current_outcome = &self.solver.game.outcomes()[self.outcome_index];
                    Some(Position {
                        solver: self.solver,
                        slot_mask: self.slot_mask | slot.slot_mask(),
                        move_number: 0,
                        outcome_index: self.solver.game.random_initial_outcome().index(),
                        score: self.score + self.solver.game.score(slot, current_outcome)
                    })
                },
                BestMove::Move(move_index) => {
                    let probabilities = self.solver.game.move_probabilities(self.outcome_index, move_index);
                    let idx = random_index_from_probabilities(probabilities.iter().map(|(_, p)| p));
                    Some(Position {
                        solver: self.solver,
                        slot_mask: self.slot_mask,
                        move_number: self.move_number + 1,
                        outcome_index: probabilities[idx].0,
                        score: self.score
                    })
                },
                BestMove::Unknown => None
            };
        } else {
            None
        }
    }


    fn get_best_move(&self) -> Option<BestMoveWithScore> {
        self.solver.get_best_move(self.slot_mask, self.move_number, self.outcome_index)
    }
}

impl <'a, G: Game<O, M, S>, O: Outcome, M: Move, S: Slot> std::fmt::Display for Position<'a, G, O, M, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let best_move: String = match self.get_best_move() {
            Some(BestMoveWithScore { best_move, expected_score }) => match best_move {
                BestMove::SelectSlot(slot_index) => format!("Fill slot {} with expected score {}", self.solver.game.slots()[slot_index], expected_score),
                BestMove::Move(move_index) => format!("Reroll {} with expected score {}", self.solver.game.moves()[move_index], expected_score),
                BestMove::Unknown => format!("Unknown best move with score {}!", expected_score),
            },
            None => "Unknown position!".to_string()
        };
        write!(f, "Filled slots: {}\nAvailable slots: {}\nMove#: {}\nOutcome: {}\nScore: {}\nBest move: {}\n",
            self.filled_slots().iter().map(|s| s.to_string()).collect::<Vec<String>>().join(","),
            self.available_slots().iter().map(|s| s.to_string()).collect::<Vec<String>>().join(","),
            self.move_number,
            self.solver.game.outcomes()[self.outcome_index],
            self.score,
            best_move
        )
    }
}











/*
Subproblem:
given list of expected score per possible outcome, find the expected score for the round.

Input: per possible outcome [(expected_score: f32, slot_selection_move_index: usize)]


// For each number of re-rolls per round
//
// Input from "previous" re-roll
let input_expected_score_per_outcome: [f32];
//
// Calculate a new list per outcome [(expected_score: f32, re_roll_move_index: usize)] by
let output_expected_score_per_outcome: [f32];
for from in outcomes {
    let mut best: Option<(f32, MoveIndex)> = None;
    for mov in moves {
        let mut expected_score = 0.0;
        for (p, to_outcome_index) in forward_probabilities(from, mov) {
            expected_score += expected_score_per_outcome[to_outcome_index] * p;
        }
        if best == None || best.0 < expected_score {
            best = (expected_score, mov);
        }
    }
    output_expected_score_per_outcome[from_index] = best;
}

After all re-rolls we are at the initial position of the round. The best expected score for the round
is then the probability of each initial outcome times the expected score for that outcome.

In the end we have a round with available slots, a list of expected scores per outcome for each re-roll
of the round with the optimal move (slot-selection or re-roll) for that outcome. We also have the
optimal expected (remaining) score at the start of the round.

The rounds from previous stage can then be used to find the initial values for the rounds in the next
stage by for each round in previous stage unfilling one slot and adding the value of that slot for each
possible final outcome to the expected score (thus generating the first input). Note that multiple
rounds from the previous stage can be candidates for the same outcome in a round in the next stage. The
one resulting in best expected score should be used.

Each round can be "solved" individually, and can thus be parallelized. Between stages we need the total
solved rounds to setup the next stage.
*/
