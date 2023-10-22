mod dice_game;
//use dice_game::abstract_game::*;
use dice_game::game_solver::*;
use dice_game::game_player::*;
use dice_game::*;

use std::io;
use std::time::{Instant};


fn print_outcome_probabilities(v: Vec<(&UnorderedDiceOutcome, f32)>) -> String {
    v.iter().map(|(o, p)| format!("[{}: {:.2}%]", &o, (*p)*100.0)).collect::<Vec<String>>().join(",")
}

//const bonus_factor: f32 = 50.0/63.0;
const bonus_factor: f32 = 2.0;

fn main() {
    println!("Generating game...");
    // 3 of each in cat ones to sixes gives bonus = 3*(1+2+3+4+5+6)=63
    // Spread out the 50 bonus points on the on the remaining 2*(1+2+3+4+5+6)=42 points,
    // This equals 50/42 points extra for each pip exceeding 3 of that kind.
    // Examples:
    //   four 4's = 4*4 + 1*4*50/42 = 20.76 points
    //   three 3's = 3*3 = 9 points
    //   five 6's = 5*6 + 2*6*50/42 = 44.28 points
    let slots: Vec<DiceSlotDescription> = vec![
         dice_game::DiceSlotDescription::ones(Some(|p| if p <= 2.0 { p } else { p + (p-2.0)*bonus_factor }))
         , dice_game::DiceSlotDescription::twos(Some(|p| if p <= 4.0 { p } else { p + (p-4.0)*bonus_factor }))
         , dice_game::DiceSlotDescription::threes(Some(|p| if p <= 6.0 { p } else { p + (p-6.0)*bonus_factor }))
         , dice_game::DiceSlotDescription::fours(Some(|p| if p <= 8.0 { p } else { p + (p-8.0)*bonus_factor }))
         , dice_game::DiceSlotDescription::fives(Some(|p| if p <= 10.0 { p } else { p + (p-10.0)*bonus_factor }))
         , dice_game::DiceSlotDescription::sixes(Some(|p| if p <= 12.0 { p } else { p + (p-12.0)*bonus_factor }))
        //  dice_game::DiceSlotDescription::ones(None)
        //  , dice_game::DiceSlotDescription::twos(None)
        //  , dice_game::DiceSlotDescription::threes(None)
        //  , dice_game::DiceSlotDescription::fours(None)
        //  , dice_game::DiceSlotDescription::fives(None)
        //  , dice_game::DiceSlotDescription::sixes(None)
         , dice_game::DiceSlotDescription::one_pair()
         , dice_game::DiceSlotDescription::two_pairs()
         , dice_game::DiceSlotDescription::three_of_a_kind()
         , dice_game::DiceSlotDescription::four_of_a_kind()
         , dice_game::DiceSlotDescription::small_straight()
         , dice_game::DiceSlotDescription::large_straight()
         , dice_game::DiceSlotDescription::full_house()
         , dice_game::DiceSlotDescription::yatzy()
         , dice_game::DiceSlotDescription::chance()
    ];
    let num_dice = 5;
    let num_sides = 6;
    let num_rolls = 3;
    let bonus_threshold: f32 = 63.0;
    let bonus_score: f32 = 50.0;
    let game = dice_game::DiceGame::new(num_dice, num_sides, num_rolls, bonus_threshold, bonus_score, slots);
    println!("Game: {}", game);

    // let outcome_vec = vec![1, 1, 1];
    // let move_vec = vec![false, true, true];

    // if let (Some(outcome), Some(mov)) = (game.find_outcome(&outcome_vec), game.find_move(&move_vec)) {
    //     println!("Outcome: {}", outcome);
    //     println!("Reroll: {}", mov);
    //     let v = game.move_probabilities(outcome.index(), mov.index()).iter().map(|(o_idx, p)| (&game.outcomes()[*o_idx], *p)).collect();
    //     println!("Forward move probabilities {} -> {} -> x: {}", outcome, mov, print_outcome_probabilities(v));
    // }

    let mut solver: GameSolver<'_, DiceGame, UnorderedDiceOutcome, DiceReroll, DiceSlot> = dice_game::game_solver::GameSolver::new(&game);

    let start = Instant::now();
    solver.solve();
    let duration = start.elapsed();
    println!("Time to solve game is: {:?}", duration);

    println!("=======================================\n");

//        play_game(&solver.initial_position(None));

//        let protocol = dice_game::game_player::play_whole_game(&solver);
//        println!("Protocol {}: {:?}", i, protocol);

    let statistics = play_games(&solver, 1000000);
    println!("Statistics: {}", statistics);

    statistics.print_histogram();
}

/*
fn play_game<'a>(position: &Position<'a, DiceGame, UnorderedDiceOutcome, DiceReroll, DiceSlot>) {
    println!("Current position\n================\n{}", position);

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("error: unable to read user input");

    if let Some(next_position) = position.follow_best_move() {
        play_game(&next_position);
    } else {
        println!("Reached end of play");
    }
}
*/

/*
State
===========
Filled slots (u16)
Optimal expected remaining score (u16)
After dice roll # (u8)
Current outcome (Outcome)
Map move => summed expected score (add probability of move * expected score)

After all new states have been calculated (width first), for each new state choose the move with highest expected score
The optimal expected remaining score is that highest expected score

State transition
================
After dice roll > 1:
After dice roll -= 1
Current outcome -> <all possible moves -> possible outcomes>
Link to current + probability

After dice roll == 1:
After dice roll = num dice rolls
Selected slot = <any/all of the filled slots>
Filled slots -= <selected slot>
Expected score += current outcome(score for selected slot)
Current outcome = <all possible outcomes>
*/