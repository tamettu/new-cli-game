use std::thread;
use std::time::Duration;

use minifb::Icon::Buffer;
use minifb::Window;
use minifb::WindowOptions;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;

use crate::config;
use crate::creature::Creature;
use crate::creature::Gender;
use crate::creature::Gender::Nonbinary;
use crate::creature::Specie;
use crate::creature::TrophicLevel;
use crate::creature::TrophicLevel::Producer;
use crate::game_tools;
use crate::game_tools::CoorUsize;
use crate::game_tools::log_tamettu;
use crate::game_tools::logln_tamettu;
use crate::game_tools::logln_tamettu_clean;
use crate::gen_map;
#[derive(Copy, Clone, Debug)]
pub enum Movement {
    Eat,
    Atk,
    Walk,
    Mating,
    Gone,
}
#[derive(Copy, Clone)]
pub struct ActorMovement {
    pub actor_pos: CoorUsize,
    pub action: Movement,
    pub target_pos: Option<CoorUsize>,
}
impl ActorMovement {
    pub fn new(actor_pos: CoorUsize, action: Movement, target_pos: Option<CoorUsize>) -> Self {
        ActorMovement {
            actor_pos,
            action,
            target_pos,
        }
    }
}
pub struct SimulationMap {
    pub sim_map: gen_map::GameMap,
    creatures: Vec<Option<Creature>>,
    species: Vec<Specie>,
    pub buffer: Vec<u32>,
}
impl SimulationMap {
    pub fn new(width: usize, height: usize, species: Vec<Specie>) -> Self {
        let sim_map = gen_map::GameMap::new(width, height);
        let creatures = vec![None; width * height];
        let buffer = vec![0x00401624; width * height];
        SimulationMap {
            sim_map,
            creatures,
            species,
            buffer,
        }
    }
    pub fn show_gui(&self, window: &mut Window) {
        window
            .update_with_buffer(&self.buffer, self.sim_map.width, self.sim_map.height)
            .unwrap();
    }
    fn is_target_out_of_map(&self, target_pos: CoorUsize) -> bool {
        target_pos.x >= self.sim_map.width || target_pos.y >= self.sim_map.height
    }
    fn is_target_empty(&self, target_pos: CoorUsize) -> bool {
        self.creatures[target_pos.to_index(self.sim_map.width)].is_none()
    }
    pub fn new_creature(&mut self, target: Creature) -> Result<(), ()> {
        if self.is_target_out_of_map(target.pos) || !self.is_target_empty(target.pos) {
            return Err(());
        }
        self.creatures[target.pos.to_index(self.sim_map.width)] = Some(target);
        self.sim_map.change_str_map_val(
            &target.pos,
            &self.species[target.sid as usize].icon.to_string(),
        );
        self.buffer[target.pos.to_index(self.sim_map.width)] =
            self.species[target.sid as usize].color_for_gui;
        Ok(())
    }
    fn change_coor(&mut self, actor: Creature, target_pos: CoorUsize) {
        if self.is_target_out_of_map(target_pos) {
            return;
        }
        self.del_creature(actor.pos);
        let mut new_creature = actor;
        new_creature.pos = target_pos;
        let _ = self.new_creature(new_creature);
    }
    fn del_creature(&mut self, target_pos: CoorUsize) {
        if self.is_target_out_of_map(target_pos) {
            return;
        }
        self.sim_map
            .change_str_map_val(&target_pos, config::EMPTY_ICON);
        self.creatures[target_pos.to_index(self.sim_map.width)] = None;
        self.buffer[target_pos.to_index(self.sim_map.width)] = 0x00401624;
    }
    fn get_creatures_only(&self) -> Vec<&Creature> {
        self.creatures.par_iter().flatten().collect()
    }
    fn atk(&mut self, actor_pos: CoorUsize, target_pos: CoorUsize) {
        let Some(target) = self.creatures[target_pos.to_index(self.sim_map.width)].as_ref() else {
            return;
        };
        let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)].as_ref() else {
            return;
        };
        if actor.energy_left == 0 || target.hp_left == 0 {
            return;
        }
        let damage = (self.species[target.sid as usize].max_hp as f32 * 0.4) as usize + 1;
        if let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)].as_mut() {
            actor.energy_left -= 1;
        }
        if let Some(target) = self.creatures[target_pos.to_index(self.sim_map.width)].as_mut() {
            target.hp_left = target.hp_left.saturating_sub(damage);
        }
    }
    fn walk(&mut self, actor_pos: CoorUsize, new_pos: CoorUsize) {
        if self.is_target_out_of_map(new_pos) || !self.is_target_empty(new_pos) {
            return;
        }
        let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)] else {
            return;
        };
        if actor.energy_left == 0 {
            return;
        }
        self.change_coor(actor, new_pos);
    }
    fn eat(&mut self, actor_pos: CoorUsize, target_coor: CoorUsize) {
        let Some(target) = self.creatures[target_coor.to_index(self.sim_map.width)] else {
            return;
        };
        let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)].as_ref() else {
            return;
        };
        if actor.energy_left == 0 {
            return;
        }
        let total_energy_get = (target.energy_left as f32
            * (0.1_f32.powf(
                (self.species[actor.sid as usize].trophic_level as u8
                    - self.species[target.sid as usize].trophic_level as u8) as f32,
            )))
        .round() as usize;
        if let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)].as_mut() {
            actor.energy_left += total_energy_get;
            if actor.energy_left > self.species[actor.sid as usize].max_energy {
                actor.energy_left = self.species[actor.sid as usize].max_energy;
            }
        }
        if let Some(target) = self.creatures[target_coor.to_index(self.sim_map.width)].as_mut() {
            target.energy_left = target
                .energy_left
                .saturating_sub(self.species[target.sid as usize].max_energy / 10);
        }
    }
    fn mating(&mut self, actor_pos: CoorUsize, target_pos: CoorUsize) {
        if self.is_target_out_of_map(target_pos) || !self.is_target_empty(target_pos) {
            return;
        }
        let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)] else {
            return;
        };
        if let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)].as_mut() {
            if self.species[actor.sid as usize].trophic_level == Producer {
                actor.energy_left = actor.energy_left.saturating_sub(1)
            } else {
                actor.energy_left = actor
                    .energy_left
                    .saturating_sub((actor.energy_left as f32 / 1.5).round() as usize);
            }
            actor.mating_cold_down = self.species[actor.sid as usize].max_mating_cold_down;
        }
        let _ = self.new_creature(Creature::new(
            &self.species[actor.sid as usize],
            target_pos,
            if actor.gender == Nonbinary {
                Nonbinary
            } else {
                match rand::random_range(0_u8..2) {
                    0 => Gender::Female,
                    1 => Gender::Male,
                    _ => Gender::Female,
                }
            },
        ));
    }
    fn update(&mut self, ref_species: &Vec<&Specie>) {
        self.creatures
            .par_iter_mut()
            .flatten()
            .for_each(|c| c.update(&ref_species, &self.sim_map));
        let mut creatures = self.get_creatures_only();
        creatures.sort_unstable_by_key(|&c| c.pos);
        let mut actions: Vec<ActorMovement> = Vec::with_capacity(creatures.len());
        actions.extend(
            creatures
                .iter()
                .filter_map(|&&c| c.get_movement_leader(&ref_species, &self.sim_map, &creatures)),
        );

        // ======================================= //

        let _ = actions.iter().for_each(|c| {
            game_tools::debug_tamettu(&format!(
                "{} {} {}| {:?} | {} {}\n",
                if let Some(a) = self.creatures[c.actor_pos.to_index(self.sim_map.width)] {
                    self.species[a.sid as usize].icon.to_string()
                } else {
                    "None".to_string()
                },
                c.actor_pos.x,
                c.actor_pos.y,
                c.action,
                if let Some(a) = c.target_pos {
                    a.x as isize
                } else {
                    -1
                },
                if let Some(a) = c.target_pos {
                    a.y as isize
                } else {
                    -1
                }
            ));
            match c.action {
                Movement::Eat => {
                    self.eat(c.actor_pos, c.target_pos.unwrap());
                }
                Movement::Atk => {
                    self.atk(c.actor_pos, c.target_pos.unwrap());
                }
                Movement::Walk => {
                    self.walk(c.actor_pos, c.target_pos.unwrap());
                }
                Movement::Mating => {
                    self.mating(c.actor_pos, c.target_pos.unwrap());
                }
                Movement::Gone => {
                    self.del_creature(c.actor_pos);
                }
                _ => (),
            }
        });
    }
}

pub fn main(
    map_size: CoorUsize,
    species: &Vec<Specie>,
    init_animals_amount: &[usize],
    rest_time: Duration,
    how_long_log_once: usize,
) {
    game_tools::debug_override_tamettu("");
    let ref_species: Vec<&Specie> = species.iter().collect();
    let mut sim_map = SimulationMap::new(map_size.x, map_size.y, species.clone());
    let mut window = Window::new(
        "Little life sim",
        sim_map.sim_map.width,
        sim_map.sim_map.height,
        {
            let mut opts = WindowOptions::default();
            opts.scale_mode = minifb::ScaleMode::AspectRatioStretch;
            opts.scale = minifb::Scale::X2;
            opts.resize = true;
            opts
        },
    )
    .unwrap();
    for i in 0..species.len() {
        for _ in 0..init_animals_amount[i] {
            loop {
                if sim_map
                    .new_creature(Creature::new(
                        &species[i],
                        CoorUsize::new(
                            rand::random_range(0..sim_map.sim_map.width - 1),
                            rand::random_range(0..sim_map.sim_map.height - 1),
                        ),
                        if species[i].trophic_level == TrophicLevel::Producer {
                            Gender::Nonbinary
                        } else {
                            match rand::random_range(0..2) {
                                0 => Gender::Female,
                                1 => Gender::Male,
                                _ => Gender::Nonbinary,
                            }
                        },
                    ))
                    .is_err()
                {
                    continue;
                } else {
                    break;
                }
            }
        }
    }
    let mut log_counter = 0;
    logln_tamettu_clean("log.txt");
    loop {
        log_counter += 1;
        if log_counter == how_long_log_once {
            log_counter = 0;
            let mut species_amount: Vec<usize> = vec![0; species.len()];
            sim_map
                .get_creatures_only()
                .iter()
                .for_each(|cs| species_amount[cs.sid as usize] += 1);
            species_amount
                .iter()
                .enumerate()
                .for_each(|(index, amount)| {
                    log_tamettu(
                        "log.txt",
                        &format!("{} {}|", species[index].trophic_level as usize, amount),
                    )
                });
            logln_tamettu("log.txt", "");
        }
        // sim_map.sim_map.show_map_without_case();
        sim_map.show_gui(&mut window);
        thread::sleep(rest_time);
        sim_map.update(&ref_species);
    }
}
