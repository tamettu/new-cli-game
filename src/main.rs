use std::{fs::File, io::BufReader};

use crate::{
    creature::{DietaryCategories, Specie, TrophicLevel},
    game_tools::CoorUsize,
};
mod config;
mod creature;
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
    let mut id_generator = game_tools::IdGenerator::new();
    let species = vec![
        Specie::new(
            12,
            3,
            350,
            43,
            5,
            7,
            22,
            '󰤇',
            TrophicLevel::Primary,
            DietaryCategories::Herbivorous,
            &mut id_generator,
        ),
        Specie::new(
            30,
            5,
            970,
            65,
            6,
            2,
            13,
            '',
            TrophicLevel::Secondary,
            DietaryCategories::Carnivorous,
            &mut id_generator,
        ),
        Specie::new(
            20,
            0,
            2000,
            179,
            0,
            0,
            0,
            '',
            TrophicLevel::Producer,
            DietaryCategories::NotApplicable,
            &mut id_generator,
        ),
    ];
    little_life_sim::main(CoorUsize::new(20, 15), &species, &[7, 5, 3]);
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
    // little_life_sim::main(10, 10);
}
