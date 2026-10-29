use crate::game_tools::{
    CoorIsize, CoorUsize,
    OptionGame::{self, CellCoor, Continue},
};
use crate::{
    config, gen_map,
    get_input::{self, RawGuard, get_input_with_enter},
    menu,
};
use OptionGame::{Close, Quit, Stop};
use crossterm::event::KeyEventKind;
use crossterm::{event::KeyCode, terminal};
use rust_decimal::{Decimal, prelude::FromPrimitive};
#[allow(unused_imports)]
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{BufReader, Write as _, stdout},
    path::PathBuf,
};
use std::{
    thread,
    time::{Duration, Instant},
};
//------ --------- ------- --- is_bot_mode ----- ------------ ---------- -----
fn is_bot_mode(rg: &RawGuard) -> bool {
    print!("\x1b[1J\x1b[1H");
    match menu::show_menu(
        rg,
        &["yes", "no"],
        &"\x1b[30m\x1b[43mDo you want to play with bot?\x1b[0m",
        95,
        45,
        30,
        46,
    ) {
        a if a == "yes".to_string() => true,
        a if a == "no".to_string() => false,
        _ => panic!("u ass, how the hack you even break my program"),
    }
}
//------ ------------ ---- bot AI Brain---- ----- ------------ ------------ -------
fn find_the_best_move(
    dir: &[CoorIsize],
    self_coor_map: &mut HashSet<CoorUsize>,
    self_icon: &str,
    enemy_coor_map: &mut HashSet<CoorUsize>,
    enemy_icon: &str,
    map: &mut gen_map::GameMap,
    win_need_how_long: usize,
    atk_value: Decimal,
    def_almost_win_value: Decimal,
    def_might_win_value: Decimal,
    think_time: usize,
    neurotic_rate: usize,
) -> OptionGame {
    //------ ------------ ---- Update Display---- ----- ------------ ------------ -------
    // map.show_map(map_len);
    // print!(
    //     "\x1b[1H\x1b[{}C\x1b[33m*Your little Bot is Thinking*\x1b[0m\r\x1b[{}H",
    //     map.width + 1 + map.width,
    //     map.height + 1
    // );
    // stdout().flush().unwrap();
    //------ ------------ ---- Init---- ----- ------------ ------------ -------
    fn time_sleep(start_thinking: &Instant, think_time: usize) {
        let end_thinking = start_thinking.elapsed();
        match think_time.checked_sub(end_thinking.as_millis() as usize) {
            Some(a) => {
                thread::sleep(Duration::from_millis(a as u64));
            }
            _ => {
                ();
            }
        }
    }
    let start_thinking = Instant::now();
    let mut mark_of_most_point: HashMap<CoorUsize, Decimal> = HashMap::new();
    let result = return_will_win_pos(dir, self_coor_map, self_icon, map, win_need_how_long, false);
    //------ ------------ ---- Check Bot possible win place ---- ----- ------------ ------------ -------
    if !result.is_empty() {
        for r in result.iter() {
            self_coor_map.insert(r.clone()); //CHANGED, need change back !!!!!!!
            if !is_win(dir, &self_coor_map, self_icon, map, win_need_how_long) {
                self_coor_map.remove(&r); //CHANGED back !!!!
                let _ = *mark_of_most_point
                    .entry(r.clone())
                    .or_insert(Decimal::from_f32(0.0).unwrap()) += atk_value;
                continue;
            }
            time_sleep(&start_thinking, think_time);
            //------ ------------ ---- Update Display2 ---- ----- ------------ ------------ -------
            // let map_len = &gen_map::CLIGameLength::new(map);
            // print!(
            //     "\x1b[{}H\x1b[{}C\x1b[32m{}\x1b[0m\x1b[{}H",
            //     ((r.y + 1) * 2),
            //     ((config::WIDTH_TIMES + 1) / 2) + (config::WIDTH_TIMES + 1) * r.x,
            //     bot_icon,
            //     map_len.height + 1
            // );
            // stdout().flush().unwrap();
            // println!("Game over. You Lose....");
            return Quit;
        }
    }
    //------ ------------ ---- Check Player Possible Win Place ---- ----- ------------ ------------ -------
    let player_result = return_will_win_pos(
        dir,
        enemy_coor_map,
        enemy_icon,
        map,
        win_need_how_long,
        true,
    );
    for pr in player_result.iter() {
        enemy_coor_map.insert(pr.clone()); //CHANGED, need change back !!!!!!!
        if is_win(dir, enemy_coor_map, enemy_icon, map, win_need_how_long) {
            let _ = *mark_of_most_point
                .entry(pr.clone())
                .or_insert(Decimal::from_f32(0.0).unwrap()) += def_almost_win_value;
        }
        enemy_coor_map.remove(&pr); //CHANGED back !!!!
        let _ = *mark_of_most_point
            .entry(pr.clone())
            .or_insert(Decimal::from_f32(0.0).unwrap()) += def_might_win_value;
    }
    //------ ------------ ---- Choice the beset pos  ---- ----- ------------ ------------ -------
    // debug
    // let mut path = PathBuf::new();
    // path.push("assets");
    // path.push("AI_test.txt");
    // let mut file = OpenOptions::new()
    //     .write(true)
    //     .append(true)
    //     .open(&path)
    //     .unwrap();
    // let mut full_text: String = "---------\n".to_string();
    // for (m, k) in mark_of_most_point.iter() {
    //     let _ = write!(full_text, "{}", format!("{} {}: {}\n", m.y, m.x, k));
    // }
    // let _ = write!(file, "{}", full_text);
    if let Some(mut max_mark_k) = mark_of_most_point.keys().next() {
        if rand::random_range(0..=100) <= neurotic_rate {
            let temp_table: Vec<&CoorUsize> = mark_of_most_point.iter().map(|(k, _)| k).collect();
            return CellCoor(temp_table[rand::random_range(..temp_table.len())].clone());
        }
        let mut max_mark = Decimal::from_f32(0.0).unwrap();
        for (k, &v) in mark_of_most_point.iter() {
            if v >= max_mark {
                max_mark = v;
                max_mark_k = k;
            }
        }
        time_sleep(&start_thinking, think_time);
        OptionGame::CellCoor(max_mark_k.clone())
    } else {
        time_sleep(&start_thinking, think_time);
        CellCoor(CoorUsize::new(map.width / 2, map.height / 2))
    }
}
//------ ------------ ---- bot AI find all winning points ---- ----- ------------ ------------ -------
fn return_will_win_pos(
    dir: &[CoorIsize],
    coor_map: &HashSet<CoorUsize>,
    player_icon: &str,
    map: &gen_map::GameMap,
    win_need_how_long: usize,
    is_for_enemy: bool,
) -> Vec<CoorUsize> {
    fn change_value_without_out_of_boundary(
        coor: &CoorIsize,
        added_value: &CoorIsize,
        map: &gen_map::GameMap,
    ) -> Option<CoorIsize> {
        coor.add(added_value).filter(|r| {
            !(r.x < 0 || r.y < 0 || r.x >= map.width as isize || r.y >= map.height as isize)
        })
    }
    let mut will_win_point: Vec<CoorUsize> = Vec::new();
    let mut temp_will_win_point: Vec<CoorUsize> = Vec::with_capacity(4);
    for m in coor_map.iter() {
        for d in dir {
            //------ ------------ ------- --- init --- ------------ ------------ ----------
            temp_will_win_point.clear();
            let mut total_movable = 1;
            let mut almost_win_counter = 2;
            let mut z = CoorIsize::new(m.x as isize, m.y as isize);
            let mut o_z = z.clone();
            let mut is_top_cant_win = false;
            let mut is_bottom_cant_win = false;
            //------ ------------ ------- check might win -------- ---- ------------ ----------
            for _ in 1..win_need_how_long {
                if let Some(c) = change_value_without_out_of_boundary(&z, &d, map) {
                    z = c;
                    let temp_z = &CoorUsize::new(z.x as usize, z.y as usize);

                    if map.get_str_map_val(temp_z) == player_icon && !is_top_cant_win {
                        total_movable += 1;
                        almost_win_counter += 1;
                    } else if map.get_str_map_val(temp_z) == " " {
                        total_movable += 1;
                        temp_will_win_point.push(CoorUsize::new(z.x as usize, z.y as usize));
                    } else {
                        is_top_cant_win = true;
                    }
                }
                // println!(
                //     "{} {} | {} {} | {}  {}| {}",
                //     m.y, m.x, d.y, d.x, is_top_cant_win, is_bottom_cant_win, almost_win_counter
                // );

                if let Some(c) = change_value_without_out_of_boundary(
                    &o_z,
                    &CoorIsize::new(d.x * -1, d.y * -1),
                    map,
                ) {
                    o_z = c;
                    let temp_z = &CoorUsize::new(o_z.x as usize, o_z.y as usize);
                    if map.get_str_map_val(temp_z) == player_icon && !is_bottom_cant_win {
                        total_movable += 1;
                        almost_win_counter += 1;
                    } else if map.get_str_map_val(temp_z) == " " {
                        total_movable += 1;
                        temp_will_win_point.push(CoorUsize::new(o_z.x as usize, o_z.y as usize));
                    } else {
                        is_bottom_cant_win = true;
                    }
                }
                // println!(
                //     "{} {} | {} {} | {}  {}| {}",
                //     m.y, m.x, d.y, d.x, is_top_cant_win, is_bottom_cant_win, almost_win_counter
                // );
                if !(is_bottom_cant_win && is_top_cant_win)
                    && almost_win_counter >= win_need_how_long
                    && !temp_will_win_point.is_empty()
                {
                    return temp_will_win_point;
                }
            }
            if total_movable >= win_need_how_long {
                for _ in 0..if is_for_enemy { almost_win_counter } else { 0 } {
                    will_win_point.extend(temp_will_win_point.clone());
                }
            } else {
                will_win_point.extend(temp_will_win_point.clone());
            }
        }
    }
    will_win_point
}
//------ ------------ ---- ---- is_win ----- ------------ ------------ -------
fn is_win(
    dir: &[CoorIsize],
    coor_map: &HashSet<CoorUsize>,
    player_icon: &str,
    map: &gen_map::GameMap,
    win_need_how_long: usize,
) -> bool {
    //------ ------------ ------- avoid do useless things -------------- ----------
    if coor_map.len() < win_need_how_long {
        return false;
    }
    for m in coor_map.iter() {
        for d in dir {
            //------ ------------ ------- --- init --- ------------ ------------ ----------
            let mut total_move: usize = 1;
            let mut zx: isize =
                m.x.try_into()
                    .expect("\x1b[31mErr. Something get wrong when try to get zx in the fn is_win");
            let mut zy: isize =
                m.y.try_into()
                    .expect("\x1b[31mErr. Something get wrong when try to get zy in the fn is_win");
            //------ ------------ ------- check is win -------- ---- ------------ ----------
            for _ in 1..win_need_how_long {
                zx += d.x;
                zy += d.y;
                if zx < 0 || zy < 0 || zx >= map.width as isize || zy >= map.height as isize {
                    break;
                }
                if map.get_str_map_val(&CoorUsize::new(zx as usize, zy as usize)) == player_icon {
                    total_move += 1;
                } else {
                    break;
                }
            }
            //------ ------ ------- ---- return the result --- --------- ------------ ------
            if total_move == win_need_how_long {
                return true;
            }
        }
    }
    false
}
//------ ------------ ------- -------- player_move ---------- ------------ ----------
fn player_move(
    rg: &get_input::RawGuard,
    map: &mut gen_map::GameMap,
    player_icon: &str,
) -> OptionGame {
    //------ ------------ ------- ------- init ------------ ------------ ----------
    let mut curr_x = map.width / 2;
    let mut curr_y = map.height / 2;
    map.show_map();
    //------ ------ ------- ---- end of init --- ----------- ------------ ----------
    loop {
        //------ ------------ ------- detect is overlap ------------ ------------ ----------
        print!(
            "\x1b[{}H\x1b[{}C\x1b[32m{}\x1b[0m\x1b[{}H",
            ((curr_y + 1) * 2),
            ((config::WIDTH_TIMES + 1) / 2) + (config::WIDTH_TIMES + 1) * curr_x,
            player_icon,
            map.height + 1
        );
        stdout().flush().unwrap();
        //------ ------------ ------- do move things  ------------ ------------ ----------
        match get_input::get_input(rg) {
            Some(a) if a.kind != KeyEventKind::Release => match a {
                //------ -------- inside here only handle non release button ------ -------- -----
                a if a.code == KeyCode::Char('w') || a.code == KeyCode::Up => {
                    print!(
                        "\x1b[{}H\x1b[{}C{}\x1b[{}H",
                        ((curr_y + 1) * 2),
                        ((config::WIDTH_TIMES + 1) / 2) + (config::WIDTH_TIMES + 1) * curr_x,
                        map.get_str_map_val(&CoorUsize::new(curr_x, curr_y)),
                        map.height + 1
                    );
                    stdout().flush().unwrap();

                    curr_y = if curr_y == 0 {
                        map.height - 1
                    } else {
                        curr_y - 1
                    };
                }
                a if a.code == KeyCode::Char('s') || a.code == KeyCode::Down => {
                    print!(
                        "\x1b[{}H\x1b[{}C{}\x1b[{}H",
                        ((curr_y + 1) * 2),
                        ((config::WIDTH_TIMES + 1) / 2) + (config::WIDTH_TIMES + 1) * curr_x,
                        map.get_str_map_val(&CoorUsize::new(curr_x, curr_y)),
                        map.height + 1
                    );
                    stdout().flush().unwrap();
                    curr_y = if curr_y == map.height - 1 {
                        0
                    } else {
                        curr_y + 1
                    };
                }
                a if a.code == KeyCode::Char('a') || a.code == KeyCode::Left => {
                    print!(
                        "\x1b[{}H\x1b[{}C{}\x1b[{}H",
                        ((curr_y + 1) * 2),
                        ((config::WIDTH_TIMES + 1) / 2) + (config::WIDTH_TIMES + 1) * curr_x,
                        map.get_str_map_val(&CoorUsize::new(curr_x, curr_y)),
                        map.height + 1
                    );
                    stdout().flush().unwrap();

                    curr_x = if curr_x == 0 {
                        map.width - 1
                    } else {
                        curr_x - 1
                    };
                }
                a if a.code == KeyCode::Char('d') || a.code == KeyCode::Right => {
                    print!(
                        "\x1b[{}H\x1b[{}C{}\x1b[{}H",
                        ((curr_y + 1) * 2),
                        ((config::WIDTH_TIMES + 1) / 2) + (config::WIDTH_TIMES + 1) * curr_x,
                        map.get_str_map_val(&CoorUsize::new(curr_x, curr_y)),
                        map.height + 1
                    );

                    stdout().flush().unwrap();
                    curr_x = if curr_x == map.width - 1 {
                        0
                    } else {
                        curr_x + 1
                    };
                }
                //------ -------- ------ handle non move button --- ------- ------------ ----------
                a if (a.code == KeyCode::Enter || a.code == KeyCode::Char(' '))
                    && map.get_str_map_val(&CoorUsize::new(curr_x, curr_y)) == " " =>
                {
                    print!(
                        "\x1b[{}H\x1b[{}C\x1b[32m{}\x1b[0m\x1b[{}H",
                        ((curr_y + 1) * 2),
                        ((config::WIDTH_TIMES + 1) / 2) + (config::WIDTH_TIMES + 1) * curr_x,
                        " ",
                        map.height + 1
                    );

                    map.change_str_map_val(&CoorUsize::new(curr_x, curr_y), player_icon);
                    return CellCoor(CoorUsize::new(curr_x, curr_y));
                }

                a if a.code == KeyCode::Esc => return Quit,
                _ => continue,
            },
            _ => continue,
        }
    }
}
//------ ------------ ------- --- get_players_icon ------ ------------ ----------
fn get_payers_icon(rg: &RawGuard, hint: &str, lim_len: usize) -> String {
    loop {
        print!("\x1b[1J\x1b[1H");
        let _ = terminal::disable_raw_mode();
        let Some(mut input) = get_input_with_enter(&format!("{}", hint)) else {
            continue;
        };
        input = input.trim().to_string();
        if input.chars().count() > lim_len {
            continue;
        }
        let continue_or_not_menu: [&str; 2] = [
            &format!("continue with '{}'", input.trim()),
            "change another icon",
        ];
        if menu::show_menu(rg, &continue_or_not_menu, "", 95, 45, 30, 46) == continue_or_not_menu[0]
        {
            return input;
        };
    }
}
//------ ------------ ------- --------- main fn ---------- ------------ ----------
pub fn start_game(
    rg: &get_input::RawGuard,
    width: usize,
    heigth: usize,
    win_need_how_long: usize,
    think_time: usize,
    values: &AiValues,
) -> OptionGame {
    //------ ---------- ------- ----- init --- ------------ ------------ ----------
    let is_bot_mode = is_bot_mode(rg);
    let mut game_map = gen_map::GameMap::new(width, heigth);
    let mut player1_pos: HashSet<CoorUsize> = HashSet::new();
    let mut player2_pos: HashSet<CoorUsize> = HashSet::new();
    let dir: [CoorIsize; 4] = [
        CoorIsize::new(1, 0),
        CoorIsize::new(0, 1),
        CoorIsize::new(1, 1),
        CoorIsize::new(-1, 1),
    ];
    let mut total_items: usize = game_map.width * game_map.height;
    //------ --------- -------- get players icon --- ---------- ------------ ------
    let player1: &str = &get_payers_icon(
        rg,
        &format!(
            "Please enter a character icon up to {} characters long for \x1b[30m\x1b[43mfirst\x1b[0m player>",
            config::WIDTH_TIMES
        ),
        config::WIDTH_TIMES,
    );
    let player2 = if is_bot_mode {
        config::TATITO_BOT_ICON
    } else {
        &get_payers_icon(
            rg,
            &format!(
                "Please enter a character icon up to {} characters long for \x1b[30m\x1b[43msecond\x1b[0m player>",
                config::WIDTH_TIMES
            ),
            config::WIDTH_TIMES,
        )
    };
    let mut curr_player = player2;
    //------ ------------ ------- --- game loop --- ------------ ------- ------
    while total_items != 0 {
        game_map.show_map();
        curr_player = if curr_player == player2 {
            player1
        } else {
            player2
        };

        if curr_player == player2 && is_bot_mode {
            let CellCoor(best_point) = find_the_best_move(
                &dir,
                &mut player2_pos,
                player2,
                &mut player1_pos,
                player1,
                &mut game_map,
                win_need_how_long,
                values.atk_value,
                values.def_almost_win_value,
                values.def_might_win_value,
                think_time,
                5,
            ) else {
                return Quit;
            };
            if game_map.get_str_map_val(&best_point) == " " {
                player2_pos.insert(best_point.clone());
                game_map.change_str_map_val(&best_point, player2);
            }
            total_items -= 1;
            continue;
        }
        match player_move(rg, &mut game_map, curr_player) {
            CellCoor(t) => {
                total_items -= 1;
                if curr_player == player1 {
                    player1_pos.insert(t);
                    if !is_win(&dir, &player1_pos, player1, &game_map, win_need_how_long) {
                        continue;
                    }
                    game_map.show_map();
                    game_map.show_map();
                    println!("\rPlayer \x1b[30m\x1b[43m{}\x1b[0m win!", curr_player);
                    return Continue;
                } else {
                    player2_pos.insert(t);
                    if !is_win(&dir, &player2_pos, player2, &game_map, win_need_how_long) {
                        continue;
                    }
                    game_map.show_map();
                    println!("\rPlayer \x1b[30m\x1b[43m{}\x1b[0m win!", curr_player);
                    return Continue;
                }
            }
            Quit => return Quit,

            Close => {
                panic!("\x1b[32mThansk for playing my little game\nSee you next time!\x1b[0m");
            }
            Stop => {
                menu::show_menu(rg, &["Start"], "", 95, 45, 30, 46);
            }
            Continue => {
                continue;
            }
        }
    }
    println!("\rDraw! No one win...");
    return Quit;
}
#[derive(Clone, PartialEq)]
pub struct AiValues {
    pub atk_value: Decimal,
    pub def_almost_win_value: Decimal,
    pub def_might_win_value: Decimal,
}

impl AiValues {
    pub fn new(
        atk_value: Decimal,
        def_almost_win_value: Decimal,
        def_might_win_value: Decimal,
    ) -> Self {
        AiValues {
            atk_value,
            def_almost_win_value,
            def_might_win_value,
        }
    }
}

impl Default for AiValues {
    fn default() -> Self {
        AiValues {
            atk_value: Decimal::from_f32(0.0).unwrap(),
            def_almost_win_value: Decimal::from_f32(0.0).unwrap(),
            def_might_win_value: Decimal::from_f32(0.0).unwrap(),
        }
    }
}
pub fn train_ai(
    width: usize,
    heigth: usize,
    win_need_how_long: usize,
    bot_0_ai_values: &AiValues,
    bot_1_ai_values: &AiValues,
    think_time: usize,
) -> Option<u8> {
    let mut game_map = gen_map::GameMap::new(width, heigth);
    let mut bot0_pos: HashSet<CoorUsize> = HashSet::new();
    let mut bot1_pos: HashSet<CoorUsize> = HashSet::new();
    let dir: [CoorIsize; 4] = [
        CoorIsize::new(1, 0),
        CoorIsize::new(0, 1),
        CoorIsize::new(1, 1),
        CoorIsize::new(-1, 1),
    ];
    let mut total_items: usize = game_map.width * game_map.height;
    let bot0_icon = "x";
    let bot1_icon = "o";

    while total_items != 0 {
        // === Show Map!!! ===
        game_map.show_map_without_case();
        let CellCoor(best_point1) = find_the_best_move(
            &dir,
            &mut bot1_pos,
            bot1_icon,
            &mut bot0_pos,
            bot0_icon,
            &mut game_map,
            win_need_how_long,
            bot_1_ai_values.atk_value,
            bot_1_ai_values.def_almost_win_value,
            bot_1_ai_values.def_might_win_value,
            think_time,
            5,
        ) else {
            return Some(1);
        };
        bot1_pos.insert(best_point1.clone());
        game_map.change_str_map_val(&best_point1, bot1_icon);
        if total_items - 1 == 0 {
            return None;
        }
        let CellCoor(best_point0) = find_the_best_move(
            &dir,
            &mut bot0_pos,
            bot0_icon,
            &mut bot1_pos,
            bot1_icon,
            &mut game_map,
            win_need_how_long,
            bot_0_ai_values.atk_value,
            bot_0_ai_values.def_almost_win_value,
            bot_0_ai_values.def_might_win_value,
            think_time,
            50,
        ) else {
            return Some(0);
        };
        bot0_pos.insert(best_point0.clone());
        game_map.change_str_map_val(&best_point0, bot0_icon);
        total_items -= 2;
        continue;
    }
    return None;
}
