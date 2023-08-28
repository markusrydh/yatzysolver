mod dice_game;
//use dice_game::abstract_game::*;
use dice_game::game_solver::*;
use dice_game::*;

use std::io;
use std::time::{Instant};

fn print_outcome_probabilities(v: Vec<(&UnorderedDiceOutcome, f32)>) -> String {
    v.iter().map(|(o, p)| format!("[{}: {:.2}%]", &o, (*p)*100.0)).collect::<Vec<String>>().join(",")
}

fn main() {
    println!("Generating game...");
    let slots: Vec<DiceSlotDescription> = vec![
        dice_game::DiceSlotDescription::ones()
        , dice_game::DiceSlotDescription::twos()
        , dice_game::DiceSlotDescription::threes()
        , dice_game::DiceSlotDescription::fours()
        , dice_game::DiceSlotDescription::fives()
        , dice_game::DiceSlotDescription::sixes()
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
    let game = dice_game::DiceGame::new(num_dice, num_sides, num_rolls, slots);
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

    play_game(&solver.initial_position(None));
}

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