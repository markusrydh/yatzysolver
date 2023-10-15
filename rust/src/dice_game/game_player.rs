use crate::game_solver::*;
use crate::abstract_game::*;

#[derive(Debug)]
pub struct GameProtocol{
    total_slot_score : f32,
    slot_scores : Vec<f32>,
    bonus : bool,
    total_game_score : f32
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