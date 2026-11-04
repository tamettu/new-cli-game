use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufReader, ErrorKind::DirectoryNotEmpty},
    path,
    sync::Arc,
};

use rand::seq::IndexedRandom;
use rust_decimal::Decimal;
use smallvec::SmallVec;

use crate::{
    game_tools::{CoorIsize, CoorUsize, OptionTUI},
    little_life_sim::{
        Creature, DietaryCategories,
        Gender::{self, Nonbinary},
        Specie, TrophicLevel,
    },
};

mod config;
mod game_tools;
mod gen_map;
mod get_input;
mod little_life_sim;
mod menu;
mod tictactoe;
//------ ------------ ------- ------------ ------------ ------------ ----------
fn main() {
    let file = File::open("assets/tactictoe/ai_values.json").unwrap();
    let reader = BufReader::new(file);
    // let mut ai_values = AiValues::default();
    // let test: HashMap<String, Decimal> = serde_json::from_reader(reader).unwrap();
    // for (k, v) in test {
    //     match k {
    //         t0 if t0 == "atk_value".to_string() => ai_values.atk_value += v,
    //         t0 if t0 == "def_almost_win_value".to_string() => ai_values.def_almost_win_value += v,
    //         t0 if t0 == "def_might_win_value".to_string() => ai_values.def_might_win_value += v,
    //         _ => (),
    //     }
    // }

    // let rg = get_input::RawGuard::new();
    // let all_options = ["Tic-Tac-Toe", "2048", "A Little RPG Game", "quit"];
    // loop {
    //     print!("\x1b[1J");
    //     match menu::show_menu(&rg, &all_options, "", 95, 45, 30, 46).as_str() {
    //         "Tic-Tac-Toe" => {
    //             match tictactoe::start_game(&rg, 15, 15, 5, 200, &ai_values) {
    //                 OptionGame::Quit => {
    //                     return;
    //                 }
    //                 _ => return,
    //             };
    //         }
    //         "quit" => {
    //             println!("\rBye Bye");
    //             return;
    //         }
    //         _ => continue,
    //     }
    // }

    // let mut bot_0_win = 0;
    // let mut bot_0_ai_values = AiValues::default();
    // let mut bot_1_win = 0;
    // let mut bot_1_ai_values = AiValues::default();

    // let test: HashMap<String, Decimal> = serde_json::from_reader(reader).unwrap();
    // for (k, v) in test {
    //     match k {
    //         t0 if t0 == "atk_value".to_string() => {
    //             bot_0_ai_values.atk_value += v;
    //             bot_1_ai_values.atk_value += v;
    //         }
    //         t0 if t0 == "def_almost_win_value".to_string() => {
    //             bot_0_ai_values.def_almost_win_value += v;
    //             bot_1_ai_values.def_almost_win_value += v;
    //         }
    //         t0 if t0 == "def_might_win_value".to_string() => {
    //             bot_0_ai_values.def_might_win_value += v;
    //             bot_1_ai_values.def_might_win_value += v;
    //         }
    //         _ => (),
    //     }
    // }
    // fn change_a_little_bit_bot_values(bot_values: &mut AiValues, max_buff: f64, min_debuff: f64) {
    //     let Some(new_atk) = Decimal::from_f64_retain(rand::random_range(min_debuff..=max_buff))
    //     else {
    //         game_tools::debug(
    //             "failed to turn f64 -> Decimal at atk in fn change_a_little_bit_bot_values",
    //             false,
    //         );
    //         return;
    //     };
    //     let Some(new_def_almost) =
    //         Decimal::from_f64_retain(rand::random_range(min_debuff..=max_buff))
    //     else {
    //         game_tools::debug(
    //             "failed to turn f64 -> Decimal at almost win in fn change_a_little_bit_bot_values",
    //             false,
    //         );
    //         return;
    //     };
    //     let Some(new_def_might) =
    //         Decimal::from_f64_retain(rand::random_range(min_debuff..=max_buff))
    //     else {
    //         game_tools::debug(
    //             "failed to turn f64 -> Decimal at might win in fn change_a_little_bit_bot_values",
    //             false,
    //         );
    //         return;
    //     };
    //     bot_values.atk_value += new_atk;
    //     bot_values.def_almost_win_value += new_def_almost;
    //     bot_values.def_might_win_value += new_def_might;
    // }
    // let max_lose = 5;
    // let mut bot0_lose_counter = 0;
    // let mut bot1_lose_counter = 0;
    // for _ in 0..4200 {
    //     if bot0_lose_counter == max_lose {
    //         change_a_little_bit_bot_values(&mut bot_0_ai_values, 0.5, -0.5);
    //         bot0_lose_counter = 0;
    //     }
    //     if bot1_lose_counter == max_lose {
    //         change_a_little_bit_bot_values(&mut bot_1_ai_values, 0.5, -0.5);
    //         bot0_lose_counter = 0;
    //     }
    //     match tictactoe::train_ai(15, 15, 5, &bot_0_ai_values, &bot_1_ai_values, 0) {
    //         Some(a) if a == 0 => {
    //             bot1_lose_counter += 1;
    //             bot_0_win += 1;
    //         }
    //         Some(_) => {
    //             bot0_lose_counter += 1;
    //             bot_1_win += 1;
    //         }
    //         _ => (),
    //     };
    //     if bot0_lose_counter == max_lose {
    //         change_a_little_bit_bot_values(&mut bot_0_ai_values, 0.5, -0.5);
    //         bot0_lose_counter = 0;
    //     }
    //     if bot1_lose_counter == max_lose {
    //         change_a_little_bit_bot_values(&mut bot_1_ai_values, 0.5, -0.5);
    //         bot0_lose_counter = 0;
    //     }
    //     match tictactoe::train_ai(15, 15, 5, &bot_1_ai_values, &bot_0_ai_values, 0) {
    //         Some(a) if a == 0 => {
    //             bot0_lose_counter += 1;
    //             bot_1_win += 1;
    //         }
    //         Some(_) => {
    //             bot1_lose_counter += 1;
    //             bot_0_win += 1;
    //         }
    //         _ => (),
    //     };
    // }
    // println!(
    //     "{} {} {} | {}\n{} {} {} | {}",
    //     bot_0_ai_values.atk_value,
    //     bot_0_ai_values.def_almost_win_value,
    //     bot_0_ai_values.def_might_win_value,
    //     bot_0_win,
    //     bot_1_ai_values.atk_value,
    //     bot_1_ai_values.def_almost_win_value,
    //     bot_1_ai_values.def_might_win_value,
    //     bot_1_win
    // );

    let mut rng = rand::rng();
    let mut id_generator = little_life_sim::IdGenerator::new();
    let mut sim_map = gen_map::GameMap::new(10, 10);
    let mut all_species: SmallVec<[Specie; 3]> = SmallVec::new();
    let mut all_creature_data: Vec<Creature> = Vec::new();
    let mut all_empty_place: Vec<CoorUsize> = Vec::with_capacity(sim_map.width * sim_map.height);
    for i in 0..sim_map.height {
        for j in 0..sim_map.width {
            let new_coor = CoorUsize::new(j, i);
            all_empty_place.push(new_coor);
        }
    }
    all_species.push(little_life_sim::Specie::new(
        0,
        100,
        100,
        60,
        3,
        7,
        '󱩡',
        TrophicLevel::Primary,
        DietaryCategories::Herbivorous,
        &mut id_generator,
    ));
    all_species.push(little_life_sim::Specie::new(
        1,
        300,
        300,
        200,
        5,
        9,
        '',
        TrophicLevel::Secondary,
        DietaryCategories::Carnivorous,
        &mut id_generator,
    ));
    all_species.push(little_life_sim::Specie::new(
        1,
        0,
        100,
        1000,
        0,
        0,
        '󱔐',
        TrophicLevel::Producer,
        DietaryCategories::NotApplicable,
        &mut id_generator,
    ));

    // auto create Creatures

    // for s in all_species.iter() {
    //     let mut max = 10;
    //     if s.trophic_level == TrophicLevel::Producer {
    //         max = 100;
    //     } else {
    //         max = 10;
    //     }
    //     for _ in 0..max {
    //         let Some(new_coor) = all_empty_place.choose(&mut rng).cloned() else {
    //             break;
    //         };
    //         let Ok(new_creature) = Creature::new(&s, &mut sim_map, new_coor, Gender::Nonbinary)
    //         else {
    //             continue;
    //         };
    //         all_creature_data.push(new_creature);
    //         all_empty_place.retain(|c| *c != new_coor);
    //     }
    // }
    // sim_map.show_map_without_case();
    all_creature_data.push(
        Creature::new(
            &all_species[1],
            &mut sim_map,
            CoorUsize::new(3, 7),
            Nonbinary,
        )
        .unwrap(),
    );
    all_creature_data.push(
        Creature::new(
            &all_species[1],
            &mut sim_map,
            CoorUsize::new(5, 2),
            Nonbinary,
        )
        .unwrap(),
    );
    all_creature_data.push(
        Creature::new(
            &all_species[1],
            &mut sim_map,
            CoorUsize::new(1, 1),
            Nonbinary,
        )
        .unwrap(),
    );
    let poor_tester = Creature::new(
        &all_species[0],
        &mut sim_map,
        CoorUsize::new(5, 5),
        Nonbinary,
    )
    .unwrap();
    all_creature_data.push(poor_tester.clone());
    sim_map.show_map_without_case();
    let ref_creatures_data: Vec<&Creature> = all_creature_data.iter().collect();
    let ref_species_data: Vec<&Specie> = all_species.iter().collect();
    game_tools::debug("", true);
    let mut counter = 0;
    let total_loop = 10;
    // for _ in 0..total_loop {
    //     for _ in 0..100 {
    let result = poor_tester.scan_step_where(
        &ref_creatures_data,
        &ref_species_data,
        sim_map.width,
        sim_map.height,
    );
    println!("{} {}", result.x, result.y);
    //         if result != CoorIsize::new(1, 1) {
    //             counter += 1;
    //         }
    //     }
    // }
    // println!("{}", counter as f32 / total_loop as f32);
}
