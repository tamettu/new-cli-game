use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::config;
use crate::creature::Creature;
use crate::creature::Gender;
use crate::creature::Gender::Male;
use crate::creature::Gender::Nonbinary;
use crate::creature::Specie;
use crate::game_tools::CoorIsize;
use crate::game_tools::CoorUsize;
use crate::gen_map;
pub struct SimulationMap {
    pub sim_map: gen_map::GameMap,
    creatures: Vec<Option<Creature>>,
    species: Vec<Specie>,
}
impl SimulationMap {
    pub fn new(width: usize, height: usize, species: Vec<Specie>) -> Self {
        let sim_map = gen_map::GameMap::new(width, height);
        let creatures = Vec::with_capacity(width * height);
        SimulationMap {
            sim_map,
            creatures,
            species,
        }
    }
    fn get_creatures_only(&self) -> Vec<&Creature> {
        self.creatures.par_iter().flatten().collect()
    }
    pub fn get_creature(&self, target_pos: CoorUsize) -> &Option<Creature> {
        &self.creatures[target_pos.to_index(self.sim_map.width)]
    }
    pub fn walk(&mut self, actor_pos: CoorUsize, new_pos: CoorUsize) {
        let Some(Some(actor)) = self
            .creatures
            .get_mut(actor_pos.to_index(self.sim_map.width))
        else {
            return;
        };
        self.sim_map
            .change_str_map_val(&actor_pos, config::EMPTY_ICON);
        actor.energy_left -= 1;
        actor.pos = new_pos;
        self.sim_map
            .change_str_map_val(&new_pos, &self.species[actor.sid as usize].icon.to_string());
    }
    pub fn eat(&mut self, actor_pos: CoorUsize, target_coor: CoorUsize) {
        let Some(target) = self.creatures[target_coor.to_index(self.sim_map.width)] else {
            return;
        };
        let Some(Some(actor)) = self
            .creatures
            .get_mut(actor_pos.to_index(self.sim_map.width))
        else {
            return;
        };
        actor.energy_left -= 1;
        self.sim_map
            .change_str_map_val(&target_coor, config::EMPTY_ICON);
        let total_energy_get = target.energy_left
            * (0.1_f32.powf(
                (self.species[actor.sid as usize].trophic_level as u8
                    - self.species[target.sid as usize].trophic_level as u8) as f32,
            )) as usize;
        actor.energy_left += total_energy_get;
    }
    pub fn mating(
        &mut self,
        actor_pos: CoorUsize,
        all_kinds: &Vec<&CoorUsize>,
        all_species: &[&Specie],
    ) {
        let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)] else {
            return;
        };
        if actor.gender == Male {
            return;
        }
        let all_dir: [CoorIsize; 8] = [
            CoorIsize::new(0, 1),
            CoorIsize::new(0, -1),
            CoorIsize::new(1, 0),
            CoorIsize::new(-1, 0),
            CoorIsize::new(1, 1),
            CoorIsize::new(-1, -1),
            CoorIsize::new(1, -1),
            CoorIsize::new(-1, 1),
        ];
        let mut kind_dir = all_dir.len() + 1;
        let mut empty_place = None;
        for i in 0..all_dir.len() {
            let Some(new_coor) = (actor_pos.to_coor_isize() + all_dir[i])
                .and_then(|c| Some(c.to_coor_usize()))
                .filter(|c| c.x < self.sim_map.width || c.y < self.sim_map.height)
            else {
                continue;
            };
            if self.sim_map.get_str_map_val(&new_coor) == config::EMPTY_ICON {
                empty_place = Some(new_coor);
            } else if all_kinds.contains(&&new_coor) {
                kind_dir = i;
            }
        }
        if kind_dir > all_dir.len() || empty_place.is_none() {
            return;
        }
        self.creatures[actor_pos.to_index(self.sim_map.width)]
            .unwrap()
            .energy_left -= 1;
        self.creatures.push(Some(Creature::new(
            &all_species[actor.sid as usize],
            &mut self.sim_map,
            empty_place.unwrap(),
            if actor.gender == Nonbinary {
                Nonbinary
            } else {
                match rand::random_range(0_u8..2) {
                    0 => Gender::Female,
                    1 => Gender::Male,
                    _ => Gender::Female,
                }
            },
        )));
    }
}
