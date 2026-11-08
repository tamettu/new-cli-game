use crate::config::{self};
use crate::game_tools::CoorUsize;
use crate::game_tools::{self, CoorIsize};
use crate::gen_map::{self};
use crate::little_life_sim::Gender::{Male, Nonbinary};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use smallvec::SmallVec;
use std::collections::HashMap;

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
pub enum Movement {
    Eat,
    Atk,
    Walk,
    Mating,
    Dead,
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
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum DietaryCategories {
    Herbivorous,
    Carnivorous,
    Omnivorous,
    NotApplicable,
}
#[derive(Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Gender {
    Male,
    Female,
    Nonbinary,
}
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum TrophicLevel {
    Producer,
    Primary,
    Secondary,
    Top,
}
impl TrophicLevel {
    pub fn get_die_rate(&self) -> u8 {
        match self {
            TrophicLevel::Producer => 45,
            TrophicLevel::Primary => 70,
            TrophicLevel::Secondary => 84,
            TrophicLevel::Top => 15,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Specie {
    sid: u8,
    pub max_hp: usize,
    pub max_speed: usize,
    pub max_energy: usize,
    pub max_mating_cold_down: usize,
    pub detect_positive_range: usize,
    pub detect_negative_range: usize,
    pub icon: char,
    pub trophic_level: TrophicLevel,
    pub dietarycategories: DietaryCategories,
}
impl Specie {
    pub fn new(
        max_hp: usize,
        max_speed: usize,
        max_energy: usize,
        max_mating_cold_down: usize,
        detect_positive_range: usize,
        detect_negative_range: usize,
        icon: char,
        trophic_level: TrophicLevel,
        dietarycategories: DietaryCategories,
        id_generator: &mut game_tools::IdGenerator,
    ) -> Self {
        Specie {
            sid: id_generator.gen_id(),
            max_hp,
            max_speed,
            max_energy,
            max_mating_cold_down,
            detect_positive_range,
            detect_negative_range,
            icon,
            trophic_level,
            dietarycategories,
        }
    }
    fn get_sid(&self) -> u8 {
        self.sid
    }
}
#[derive(Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub struct Creature {
    pub sid: u8,
    pub pos: game_tools::CoorUsize,
    pub gender: Gender,
    pub mating_cold_down: usize,
    pub energy_left: usize,
    hp_left: usize,
    speed: usize,
    step_cd: usize,
}
impl Creature {
    pub fn new(
        specie: &Specie,
        area: &mut gen_map::GameMap,
        pos: game_tools::CoorUsize,
        gender: Gender,
    ) -> Self {
        area.change_str_map_val(&pos, &specie.icon.to_string());
        let random_debuff = |original_value: usize| -> usize {
            if original_value == 0 {
                return 0;
            }
            match rand::random_range(0_u8..3) {
                0 => (original_value as f32 * rand::random_range(0.8..=1.0)) as usize,
                1 => (original_value as f32 * rand::random_range(0.5..=0.7)) as usize,
                2 => (original_value as f32 * rand::random_range(0.0..=0.4)) as usize,
                _ => 10000000,
            }
        };
        let (
            hp_left, //jump to next line you ass lsp
            speed,
            energy_left,
            mating_cold_down,
        ) = if rand::random_range(0..=100) //jump to next line you ass lsp
            < specie.trophic_level.get_die_rate()
        {
            (0, 0, 0, 0)
        } else {
            (
                random_debuff(specie.max_hp),
                random_debuff(specie.max_speed),
                random_debuff(specie.max_energy),
                if specie.max_mating_cold_down < 1 {
                    0
                } else {
                    (specie.max_mating_cold_down as f32 //jump to next line you ass lsp
                    * rand::random_range(0.5..=1.0)) as usize
                },
            )
        };
        let sid = specie.get_sid();
        let step_cd = rand::random_range(0..3);
        Creature {
            sid,
            pos,
            gender,
            mating_cold_down,
            energy_left,
            hp_left,
            speed,
            step_cd,
        }
    }
    pub fn get_hurt(&mut self, damage: usize) {
        if self.is_dead() {
            return;
        }
        self.hp_left -= damage;
    }
    fn is_dead(&self) -> bool {
        self.hp_left == 0
    }
    fn is_target_in_range(&self, target_coor: CoorUsize, range: usize) -> bool {
        let is_y_in_range = (target_coor.y > self.pos.y && target_coor.y - self.pos.y <= range)
            || (target_coor.y <= self.pos.y && self.pos.y - target_coor.y <= range);

        let is_x_in_range = (target_coor.x > self.pos.x && target_coor.x - self.pos.x <= range)
            || (target_coor.x <= self.pos.x && self.pos.x - target_coor.x <= range);

        is_y_in_range && is_x_in_range
    }
    fn scan_area_leader<'a>(
        &self,
        map_width: usize,
        map_height: usize,
        sort_creatures: &[&'a Creature],
        range: usize,
    ) -> Option<Vec<&'a Creature>> {
        let index_sort_creatures: HashMap<CoorUsize, usize> = sort_creatures
            .iter()
            .enumerate()
            .map(|(i, &c)| (c.pos, i))
            .collect();
        let self_index = index_sort_creatures.get(&self.pos).unwrap();
        let half_range_1d = (range * map_width + range) / 3;
        let is_left_too_tight = (self_index.checked_sub(half_range_1d))
            .and_then(|c| sort_creatures.get(c))
            .and_then(|c| c.pos.to_coor_isize() * -1)
            .and_then(|c| c + self.pos.to_coor_isize())
            .and_then(|c| Some(c.abs_usize(self.pos)))
            .and_then(|c| {
                println!("{} {}", c.x, c.y);
                Some(c.x >= range / 3 && c.y >= range / 3)
            });
        let is_right_too_tight = (self_index.checked_add(half_range_1d))
            .and_then(|c| sort_creatures.get(c))
            .and_then(|c| c.pos + self.pos)
            .and_then(|c| Some(c.abs_usize(self.pos)))
            .and_then(|c| {
                println!("{} {}", c.x, c.y);
                Some(c.x >= range / 3 && c.y >= range / 3)
            });

        match (is_left_too_tight, is_right_too_tight) {
            // ========= Both left and right have stuffs =========
            (Some(a), Some(b)) => {
                if a && b {
                    game_tools::debug(&format!("a {a} b {b} | dense\n"), true);
                    self.scan_area_when_dense(map_width, map_height, sort_creatures, range)
                } else {
                    game_tools::debug(&format!("a {a} b {b} | sparse\n"), true);
                    self.scan_area_when_sparse(sort_creatures, &index_sort_creatures, range)
                }
            }
            // ========= Only left have stuffs =========
            (Some(a), None) => {
                if a {
                    game_tools::debug("a | dense\n", true);
                    self.scan_area_when_dense(map_width, map_height, sort_creatures, range)
                } else {
                    game_tools::debug("a | sparse\n", true);
                    self.scan_area_when_sparse(&sort_creatures, &index_sort_creatures, range)
                }
            }
            // ========= Only right have stuffs =========
            (None, Some(b)) => {
                game_tools::debug("b\n", true);
                if b {
                    game_tools::debug("b | dense\n", true);
                    self.scan_area_when_dense(map_width, map_height, sort_creatures, range)
                } else {
                    game_tools::debug("b | sparse\n", true);
                    self.scan_area_when_sparse(&sort_creatures, &index_sort_creatures, range)
                }
            }
            // ========= Nothings nearby =========
            _ => {
                game_tools::debug("Na\n", true);
                self.scan_area_when_sparse(&sort_creatures, &index_sort_creatures, range)
            }
        }
    }
    fn scan_area_when_sparse<'a>(
        &self,
        sort_creatures: &[&'a Creature],
        index_sort_creatures: &HashMap<CoorUsize, usize>,
        range: usize,
    ) -> Option<Vec<&'a Creature>> {
        let self_index = index_sort_creatures
            .get(&self.pos)
            .expect("Bro what the hell you didnt put your self inside!?");
        let mut l_index = *self_index;
        let mut r_index = *self_index;
        let mut all_target: Vec<&Creature> = vec![];
        loop {
            if self.is_target_in_range(sort_creatures[l_index].pos, range) {
                all_target.push(sort_creatures[l_index]);
                if let Some(a) = l_index.checked_sub(1) {
                    l_index = a;
                } else {
                    break;
                }
                continue;
            }
            break;
        }
        loop {
            if self.is_target_in_range(sort_creatures[r_index].pos, range) {
                all_target.push(sort_creatures[r_index]);
                if let Some(a) = r_index
                    .checked_add(1) //lsp duck off here
                    .filter(|&a| a < sort_creatures.len())
                {
                    r_index = a;
                } else {
                    break;
                }
                continue;
            }
            break;
        }
        if all_target.is_empty() {
            None
        } else {
            Some(all_target)
        }
    }
    fn scan_vertical_line<'a>(
        map_width: usize,
        curr_coor: &CoorUsize,
        detect_range: usize,
        all_creature_corr_and_creature: &HashMap<CoorUsize, &'a Creature>,
    ) -> Option<Vec<&'a Creature>> {
        let mut all_target_place: Vec<&Creature> = vec![];
        for x in 1..=detect_range {
            if let Some(top_x) = curr_coor.x.checked_sub_signed(x as isize) {
                let new_coor = CoorUsize::new(top_x, curr_coor.y);
                if let Some(&k) = all_creature_corr_and_creature.get(&new_coor) {
                    all_target_place.push(k);
                }
            }
            if let Some(bottom_x) = curr_coor
                .x
                .checked_add_signed(x as isize)
                .filter(|x| !(*x > map_width - 1))
            {
                let new_coor = CoorUsize::new(bottom_x, curr_coor.y);
                if let Some(&k) = all_creature_corr_and_creature.get(&new_coor) {
                    all_target_place.push(k);
                }
            }
        }
        if all_target_place.is_empty() {
            return None;
        }
        Some(all_target_place)
    }
    fn scan_area_when_dense<'a>(
        &self,
        map_width: usize,
        map_height: usize,
        all_creature_place: &[&'a Creature],
        range: usize,
    ) -> Option<Vec<&'a Creature>> {
        if self.speed == 0 {
            return None;
        }
        let mut all_target_place: Vec<&Creature> = vec![];
        let all_creature_corr_and_creature: HashMap<CoorUsize, &Creature> =
            all_creature_place.par_iter().map(|c| (c.pos, *c)).collect();
        if let Some(self_line) =
            Self::scan_vertical_line(map_width, &self.pos, range, &all_creature_corr_and_creature)
        {
            all_target_place.extend(self_line);
        }
        for y in 1..=range {
            if let Some(top_y) = self.pos.y.checked_sub(y) {
                let new_top_y_coor = CoorUsize::new(self.pos.x, top_y);
                if let Some(&k) = all_creature_corr_and_creature.get(&new_top_y_coor) {
                    all_target_place.push(k);
                };
                if let Some(new_coor) = Self::scan_vertical_line(
                    map_width,
                    &new_top_y_coor,
                    range,
                    &all_creature_corr_and_creature,
                ) {
                    all_target_place.extend(new_coor);
                }
            }

            if let Some(bottom_y) = self.pos.y.checked_add(y).filter(|&y| !(y > map_height - 1)) {
                let new_bottom_y_coor = CoorUsize::new(self.pos.x, bottom_y);
                if let Some(&k) = all_creature_corr_and_creature.get(&new_bottom_y_coor) {
                    all_target_place.push(k);
                }
                if let Some(new_coor) = Self::scan_vertical_line(
                    map_width,
                    &new_bottom_y_coor,
                    range,
                    &all_creature_corr_and_creature,
                ) {
                    all_target_place.extend(new_coor);
                }
            }
        }
        if all_target_place.is_empty() {
            return None;
        }
        Some(all_target_place)
    }
    fn scan_local_food(
        &self,
        all_local_creatures: &[&Creature],
        all_species: &[&Specie],
    ) -> Option<Vec<CoorUsize>> {
        let self_specie = &all_species[self.sid as usize];
        let all_food_coor: Vec<CoorUsize> = match self_specie.dietarycategories {
            DietaryCategories::Herbivorous => all_local_creatures
                .par_iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    target_specie.trophic_level == TrophicLevel::Producer
                        && self.is_target_in_range(c.pos, self_specie.detect_positive_range)
                })
                .map(|c| c.pos)
                .collect(),

            DietaryCategories::Carnivorous => all_local_creatures
                .par_iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    target_specie.trophic_level != TrophicLevel::Producer
                        && self.sid != c.sid
                        && (self_specie.trophic_level == TrophicLevel::Top
                            || target_specie.trophic_level < self_specie.trophic_level)
                        && self.is_target_in_range(c.pos, self_specie.detect_positive_range)
                })
                .map(|c| c.pos)
                .collect(),

            DietaryCategories::Omnivorous => all_local_creatures
                .par_iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    self.sid != c.sid
                        && (self_specie.trophic_level == TrophicLevel::Top
                            || target_specie.trophic_level < self_specie.trophic_level)
                        && self.is_target_in_range(c.pos, self_specie.detect_positive_range)
                })
                .map(|c| c.pos)
                .collect(),

            _ => return None,
        };
        if all_food_coor.is_empty() {
            None
        } else {
            Some(all_food_coor)
        }
    }
    fn scan_local_kind(
        &self,
        all_local_creatures: &[&Creature],
        all_species: &[&Specie],
    ) -> Option<Vec<CoorUsize>> {
        let self_specie = &all_species[self.sid as usize];
        let all_food_coor: Vec<CoorUsize> = all_local_creatures
            .par_iter()
            .filter(|c| {
                c.sid == self.sid
                && self //lsp don't move!
                .is_target_in_range(c.pos, self_specie.detect_positive_range)
            })
            .map(|c| c.pos)
            .collect();
        if all_food_coor.is_empty() {
            None
        } else {
            Some(all_food_coor)
        }
    }
    fn scan_local_threaten(
        &self,
        all_local_creatures: &[&Creature],
        all_species: &[&Specie],
    ) -> Option<Vec<CoorUsize>> {
        let self_specie = &all_species[self.sid as usize];
        let all_food_coor: Vec<CoorUsize> = all_local_creatures
            .par_iter()
            .filter(|c| {
                let target_specie = &all_species[c.sid as usize];
                (target_specie.trophic_level > self_specie.trophic_level
                    || (target_specie.trophic_level == TrophicLevel::Top
                        && target_specie.trophic_level == self_specie.trophic_level))
                    && self.is_target_in_range(c.pos, self_specie.detect_negative_range)
            })
            .map(|c| c.pos)
            .collect();
        if all_food_coor.is_empty() {
            None
        } else {
            Some(all_food_coor)
        }
    }

    fn scan_step_where(
        &self,
        sorted_all_creature_place: &[&Creature],
        all_species: &[&Specie],
        map_width: usize,
        map_height: usize,
    ) -> CoorIsize {
        let longer_range = if all_species[self.sid as usize].detect_positive_range
            > all_species[self.sid as usize].detect_negative_range
        {
            all_species[self.sid as usize].detect_positive_range
        } else {
            all_species[self.sid as usize].detect_negative_range
        };
        let Some(all_local_creatures) = self.scan_area_leader(
            map_width,
            map_height,
            sorted_all_creature_place,
            longer_range,
        ) else {
            return CoorIsize::new(
                rand::random_range(-1_i8..2_i8) as isize,
                rand::random_range(-1_i8..2_i8) as isize,
            );
        };
        let all_food = self.scan_local_food(&all_local_creatures, all_species);
        let all_kinds = self.scan_local_kind(&all_local_creatures, all_species);
        let all_enemies = self.scan_local_threaten(&all_local_creatures, all_species);
        let mut good_weight: usize = 0;
        let mut bad_weight: usize = 0;
        if let Some(have_food) = &all_food {
            good_weight += have_food.len();
        }
        if let Some(have_kind) = &all_kinds {
            good_weight += have_kind.len();
        }
        if let Some(have_enemy) = &all_enemies {
            bad_weight = have_enemy.len();
        }
        good_weight *= all_species[self.sid as usize].detect_positive_range;
        bad_weight *= all_species[self.sid as usize].detect_negative_range;
        match rand::random_range(0..=(good_weight + bad_weight)) {
            result if result <= good_weight => {
                let closest_food: Option<CoorUsize> = (self.energy_left
                    <= all_species[self.sid as usize].max_energy / 3)
                    .then_some(all_food)
                    .flatten()
                    .and_then(|cs| cs.par_iter().map(|c| c.abs_usize(self.pos)).min());

                let closest_kind: Option<CoorUsize> = (self.mating_cold_down == 0)
                    .then_some(all_kinds)
                    .flatten()
                    .and_then(|cs| cs.par_iter().map(|c| c.abs_usize(self.pos)).min());
                match (closest_food, closest_kind) {
                    (Some(a), Some(b)) => {
                        if a > b {
                            self.pos.vector_to(a.to_coor_isize())
                        } else {
                            self.pos.vector_to(b.to_coor_isize())
                        }
                    }
                    (Some(a), None) => self.pos.vector_to(a.to_coor_isize()),
                    (None, Some(b)) => self.pos.vector_to(b.to_coor_isize()),
                    _ => CoorIsize::new(
                        rand::random_range(-1_i8..2_i8) as isize,
                        rand::random_range(-1_i8..2_i8) as isize,
                    ),
                }
            }
            result if result > good_weight => {
                let Some(enemies) = all_enemies else {
                    return CoorIsize::new(
                        rand::random_range(-1_i8..2_i8) as isize,
                        rand::random_range(-1_i8..2_i8) as isize,
                    );
                };
                let mut all_vector = CoorIsize::new(0, 0);
                for i in enemies {
                    if let Some(new_coor) = i.vector_to(self.pos.to_coor_isize()) + all_vector {
                        all_vector = new_coor
                    }
                }

                let final_move = CoorIsize::new(
                    ((all_vector.x) > (0)) as isize - ((all_vector.x) < (0)) as isize,
                    ((all_vector.y) > (0)) as isize - ((all_vector.y) < (0)) as isize,
                );
                if final_move == CoorIsize::new(0, 0) {
                    CoorIsize::new(
                        rand::random_range(-1..2_i8) as isize,
                        rand::random_range(-1..2_i8) as isize,
                    )
                } else {
                    final_move
                }
            }
            _ => CoorIsize::new(
                rand::random_range(-1_i8..2_i8) as isize,
                rand::random_range(-1_i8..2_i8) as isize,
            ),
        }
    }
    pub fn update(
        &mut self,
        all_species: &[&Specie],
        map: &gen_map::GameMap,
        sort_creatures: &[&Creature],
    ) -> Option<Movement> {
        if self.is_dead() {
            return None;
        }
        if self.step_cd != 0 {
            self.step_cd -= 1;
            return None;
        }
        let hp_left_percent = 1 - (self.hp_left / all_species[self.sid as usize].max_hp) * 10;
        let energy_left_percent =
            1 - (self.energy_left / all_species[self.sid as usize].max_energy) * 10;
        self.step_cd = self.speed + hp_left_percent + energy_left_percent / 2;
        if energy_left_percent <= 2 {
            self.get_hurt(1);
        }
        if energy_left_percent >= 5 {
            self.energy_left -= 1;
            self.hp_left += 1;
        }
        // here walk or mating or other thing

        if let Some(walk_this_way) =
            (self.scan_step_where(sort_creatures, all_species, map.width, map.height)
                + self.pos.to_coor_isize())
            .and_then(|c| Some(c.to_coor_usize()))
            .filter(|c| c.x < map.width && c.y < map.height)
            .filter(|c| map.get_str_map_val(c) == config::EMPTY_ICON)
        {
            // return Some(Movement::WalkTo(walk_this_way));
        }
        todo!()
    }
}
pub fn main(
    width: usize,
    height: usize,
    // inital_peaceful_life_number: usize,
    // peaceful_life_dietary_categorie_generate_time: f32,
    // peaceful_life_dietary_categorie_generate_percentage: f32,
    // inital_hostile_life_number: usize,
) {
    let mut rng = rand::rng();
    let mut id_generator = game_tools::IdGenerator::new();
    let mut sim_map = gen_map::GameMap::new(width, height);
    let mut all_species: SmallVec<[Specie; 3]> = SmallVec::new();
    let mut all_creature_data: Vec<Creature> = Vec::new();
    let mut all_empty_place: Vec<CoorUsize> = Vec::with_capacity(sim_map.width * sim_map.height);
    for i in 0..sim_map.height {
        for j in 0..sim_map.width {
            let new_coor = CoorUsize::new(j, i);
            all_empty_place.push(new_coor);
        }
    }
    all_species.push(Specie::new(
        0,
        100,
        100,
        60,
        3,
        3,
        '󱩡',
        TrophicLevel::Primary,
        DietaryCategories::Herbivorous,
        &mut id_generator,
    ));
    all_species.push(Specie::new(
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
    all_species.push(Specie::new(
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
}
