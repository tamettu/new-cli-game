use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelExtend;
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
pub enum Movement {
    Eat,
    Atk,
    Walk,
    Mating,
    Gone,
}
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
    fn is_target_out_of_map(&self, target_pos: CoorUsize) -> bool {
        target_pos.x >= self.sim_map.width || target_pos.y >= self.sim_map.height
    }
    fn is_target_empty(&self, target_pos: CoorUsize) -> bool {
        self.creatures[target_pos.to_index(self.sim_map.width)].is_none()
    }

    fn change_map(&mut self, actor: Creature, target_pos: CoorUsize, icon: &str) {
        if self.is_target_out_of_map(actor.pos) || self.is_target_out_of_map(target_pos) {
            return;
        }
        self.sim_map
            .change_str_map_val(&actor.pos, config::EMPTY_ICON);
        self.creatures[actor.pos.to_index(self.sim_map.width)] = None;

        self.sim_map.change_str_map_val(&target_pos, icon);
        self.creatures[target_pos.to_index(self.sim_map.width)] = Some(actor);
    }
    fn get_creatures_only(&self) -> Vec<&Creature> {
        self.creatures.par_iter().flatten().collect()
    }
    pub fn get_creature(&self, target_pos: CoorUsize) -> &Option<Creature> {
        &self.creatures[target_pos.to_index(self.sim_map.width)]
    }
    fn atk(&mut self, actor_pos: CoorUsize, target_pos: CoorUsize) {
        let Some(target) = self.creatures[target_pos.to_index(self.sim_map.width)].clone() else {
            return;
        };
        let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)].as_ref() else {
            return;
        };
        if actor.energy_left == 0 || target.hp_left == 0 {
            return;
        }
        self.creatures[actor_pos.to_index(self.sim_map.width)]
            .as_mut()
            .unwrap()
            .energy_left -= 1;
        let damage = (self.species[target.sid as usize].max_hp as f32 * 0.1) as usize + 1;
        let new_target_hp = target.hp_left.saturating_sub(damage);
        self.creatures[target_pos.to_index(self.sim_map.width)]
            .as_mut()
            .unwrap()
            .hp_left = new_target_hp;
    }
    fn walk(&mut self, actor_pos: CoorUsize, new_pos: CoorUsize) {
        let Some(Some(actor)) = self
            .creatures
            .get_mut(actor_pos.to_index(self.sim_map.width))
        else {
            return;
        };
        if actor.energy_left == 0 {
            return;
        }
        self.sim_map
            .change_str_map_val(&actor_pos, config::EMPTY_ICON);
        actor.energy_left -= 1;
        actor.pos = new_pos;
        self.sim_map
            .change_str_map_val(&new_pos, &self.species[actor.sid as usize].icon.to_string());
    }
    fn eat(&mut self, actor_pos: CoorUsize, target_coor: CoorUsize) {
        let Some(target) = self.creatures[target_coor.to_index(self.sim_map.width)] else {
            return;
        };
        let Some(Some(actor)) = self
            .creatures
            .get_mut(actor_pos.to_index(self.sim_map.width))
        else {
            return;
        };
        if actor.energy_left == 0 {
            return;
        }
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
    fn mating(&mut self, actor_pos: CoorUsize, target_pos: CoorUsize) {
        let Some(actor) = self.creatures[actor_pos.to_index(self.sim_map.width)] else {
            return;
        };
        self.creatures[actor_pos.to_index(self.sim_map.width)]
            .as_mut()
            .unwrap()
            .energy_left -= 1;
        self.creatures.push(Some(Creature::new(
            &self.species[actor.sid as usize],
            &mut self.sim_map,
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
        )));
    }
    fn update(&mut self) {
        let ref_species: Vec<&Specie> = self.species.iter().collect();
        self.creatures
            .par_iter_mut()
            .flatten()
            .for_each(|c| c.update(&ref_species));
        let mut creatures = self.get_creatures_only();
        creatures.sort_unstable_by_key(|&c| c.pos);
        let mut actions: Vec<ActorMovement> = Vec::with_capacity(creatures.len());
        actions.par_extend(
            creatures
                .par_iter()
                .filter_map(|&&c| c.get_movement(&ref_species, &self.sim_map, &creatures)),
        );

        // ======================================= //

        let _ = actions.iter().for_each(|c| match c.action {
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
                self.creatures
                    .remove(c.actor_pos.to_index(self.sim_map.width));
                self.sim_map
                    .change_str_map_val(&c.actor_pos, config::EMPTY_ICON);
            }
            _ => (),
        });
    }
}
