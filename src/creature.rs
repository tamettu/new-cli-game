use crate::config::{self};
use crate::creature::Gender::Male;
use crate::creature::TrophicLevel::Producer;
use crate::game_tools::{self, CoorIsize};
use crate::game_tools::{CoorUsize, debug_tamettu};
use crate::gen_map::{self};
use crate::little_life_sim::ActorMovement;
use crate::little_life_sim::Movement::Walk;
use rand::random;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
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
    pub neurotic_rate: usize,
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
        neurotic_rate: usize,
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
            neurotic_rate,
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
    pub pos: CoorUsize,
    pub gender: Gender,
    pub mating_cold_down: usize,
    pub energy_left: usize,
    pub hp_left: usize,
    pub speed: usize,
    pub step_cd: usize,
}
impl Creature {
    pub fn new(specie: &Specie, pos: game_tools::CoorUsize, gender: Gender) -> Self {
        let random_debuff = |original_value: usize| -> usize {
            if original_value == 0 {
                return 0;
            }
            match rand::random_range(0..100) {
                a if a >= 60 => (original_value as f32 * rand::random_range(0.8..=1.0)) as usize,
                a if a >= 20 => (original_value as f32 * rand::random_range(0.5..=0.7)) as usize,
                _ => (original_value as f32 * rand::random_range(0.0..=0.4)).round() as usize,
            }
        };
        let (
            hp_left, //jump to next line you ass lsp
            speed,
            energy_left,
            mating_cold_down,
        ) = (
            random_debuff(specie.max_hp),
            random_debuff(specie.max_speed),
            random_debuff(specie.max_energy),
            if specie.max_mating_cold_down < 1 {
                0
            } else {
                (specie.max_mating_cold_down as f32 //jump to next line you ass lsp
                    * rand::random_range(1.0..=2.0)) as usize
            },
        );
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
            .and_then(|c| Some(c.x >= range / 3 && c.y >= range / 3));
        let is_right_too_tight = (self_index.checked_add(half_range_1d))
            .and_then(|c| sort_creatures.get(c))
            .and_then(|c| c.pos + self.pos)
            .and_then(|c| Some(c.abs_usize(self.pos)))
            .and_then(|c| Some(c.x >= range / 3 && c.y >= range / 3));

        match (is_left_too_tight, is_right_too_tight) {
            // ========= Both left and right have stuffs =========
            (Some(a), Some(b)) => {
                if a && b {
                    self.scan_area_when_dense(map_width, map_height, sort_creatures, range)
                } else {
                    self.scan_area_when_sparse(sort_creatures, &index_sort_creatures, range)
                }
            }
            // ========= Only left have stuffs =========
            (Some(a), None) => {
                if a {
                    self.scan_area_when_dense(map_width, map_height, sort_creatures, range)
                } else {
                    self.scan_area_when_sparse(&sort_creatures, &index_sort_creatures, range)
                }
            }
            // ========= Only right have stuffs =========
            (None, Some(b)) => {
                if b {
                    self.scan_area_when_dense(map_width, map_height, sort_creatures, range)
                } else {
                    self.scan_area_when_sparse(&sort_creatures, &index_sort_creatures, range)
                }
            }
            // ========= Nothings nearby =========
            _ => self.scan_area_when_sparse(&sort_creatures, &index_sort_creatures, range),
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
    fn scan_local_eatable_animals(
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
                    c.hp_left != 0
                        && c.pos != self.pos
                        && target_specie.trophic_level == TrophicLevel::Producer
                        && self.is_target_in_range(c.pos, self_specie.detect_positive_range)
                })
                .map(|c| c.pos)
                .collect(),

            DietaryCategories::Carnivorous => all_local_creatures
                .par_iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    c.hp_left != 0
                        && target_specie.trophic_level != TrophicLevel::Producer
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
                    c.hp_left != 0
                        && self.sid != c.sid
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
                    c.hp_left == 0
                        && target_specie.trophic_level == TrophicLevel::Producer
                        && self.is_target_in_range(c.pos, self_specie.detect_positive_range)
                })
                .map(|c| c.pos)
                .collect(),

            DietaryCategories::Carnivorous => all_local_creatures
                .par_iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    c.hp_left == 0
                        && target_specie.trophic_level != TrophicLevel::Producer
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
                    c.hp_left == 0
                        && self.sid != c.sid
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
                c.hp_left != 0
                    && c.pos != self.pos
                    && c.sid == self.sid
                    && self.is_target_in_range(c.pos, self_specie.detect_positive_range)
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
                c.hp_left != 0
                    && c.pos != self.pos
                    && (target_specie.trophic_level > self_specie.trophic_level
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
    fn get_nearby_creature(
        &self,
        target_list: &Vec<CoorUsize>,
        map: &gen_map::GameMap,
    ) -> Option<CoorUsize> {
        if self.energy_left == 0 {
            return None;
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
        let mut target_dir = None;
        for i in 0..all_dir.len() {
            let Some(new_coor) = (self.pos.to_coor_isize() + all_dir[i])
                .and_then(|c| Some(c.to_coor_usize()))
                .filter(|c| c.x < map.width && c.y < map.height)
            else {
                continue;
            };
            if target_list.contains(&new_coor) {
                target_dir = Some(new_coor);
            }
        }
        target_dir
    }
    fn get_nearby_space(&self, map: &gen_map::GameMap) -> Option<CoorUsize> {
        if self.gender == Male || self.energy_left == 0 {
            return None;
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
        let mut empty_sapce: Option<CoorUsize> = None;
        for i in all_dir.iter() {
            let Some(new_coor) = (self.pos.to_coor_isize() + *i)
                .and_then(|c| Some(c.to_coor_usize()))
                .filter(|c| c.x < map.width && c.y < map.height)
            else {
                continue;
            };
            if map.get_str_map_val(&new_coor) == config::EMPTY_ICON {
                empty_sapce = Some(new_coor)
            }
        }
        empty_sapce
    }
    fn is_able_to_eat(
        &self,
        all_food: &Vec<CoorUsize>,
        map: &gen_map::GameMap,
    ) -> Option<ActorMovement> {
        if let Some(a) = self.get_nearby_creature(all_food, map) {
            Some(ActorMovement::new(
                self.pos,
                crate::little_life_sim::Movement::Eat,
                Some(a),
            ))
        } else {
            None
        }
    }
    fn is_able_to_atk(
        &self,
        all_eatable_animals: &Vec<CoorUsize>,
        map: &gen_map::GameMap,
    ) -> Option<ActorMovement> {
        if let Some(a) = self.get_nearby_creature(all_eatable_animals, map) {
            Some(ActorMovement::new(
                self.pos,
                crate::little_life_sim::Movement::Atk,
                Some(a),
            ))
        } else {
            None
        }
    }
    fn is_able_to_mating(
        &self,
        all_kinds: &Vec<CoorUsize>,
        map: &gen_map::GameMap,
    ) -> Option<ActorMovement> {
        if self.mating_cold_down != 0 || self.gender == Male {
            return None;
        }
        if let (Some(_), Some(b)) = (
            self.get_nearby_creature(all_kinds, map),
            self.get_nearby_space(map),
        ) {
            Some(ActorMovement::new(
                self.pos,
                crate::little_life_sim::Movement::Mating,
                Some(b),
            ))
        } else {
            None
        }
    }
    pub fn update(&mut self, all_species: &[&Specie]) {
        if self.energy_left != 0 {
            self.energy_left -= 1
        }
        if self.hp_left == 0 {
            return;
        }
        if self.energy_left <= all_species[self.sid as usize].max_energy / 2 {
            self.hp_left -= 1;
        } else if self.energy_left != 0 && self.hp_left < all_species[self.sid as usize].max_hp {
            self.energy_left -= 1;
            self.hp_left += 1;
        }
        if self.step_cd == 0 {
            self.step_cd = self.speed;
        } else {
            self.step_cd -= 1;
        }
        if self.mating_cold_down != 0 {
            self.mating_cold_down -= 1;
        }
    }
    fn random_move(&self) -> ActorMovement {
        ActorMovement::new(
            self.pos,
            Walk,
            Some(CoorUsize::new(
                self.pos
                    .x
                    .saturating_add_signed(rand::random_range(-1..2_i8) as isize),
                self.pos
                    .y
                    .saturating_add_signed(rand::random_range(-1..2_i8) as isize),
            )),
        )
    }
    pub fn get_movement_leader(
        &self,
        all_species: &[&Specie],
        map: &gen_map::GameMap,
        sorted_creatures: &[&Creature],
    ) -> Option<ActorMovement> {
        if all_species[self.sid as usize].trophic_level == Producer && self.mating_cold_down == 0 {
            return Some(ActorMovement::new(
                self.pos,
                crate::little_life_sim::Movement::Mating,
                Some(CoorUsize::new(
                    rand::random_range(0..map.width),
                    rand::random_range(0..map.height),
                )),
            ));
        }
        if self.energy_left == 0 && self.hp_left == 0 {
            return Some(ActorMovement::new(
                self.pos,
                crate::little_life_sim::Movement::Gone,
                None,
            ));
        }
        if self.hp_left == 0 || self.step_cd != 0 || self.energy_left == 0 || self.speed == 0 {
            return None;
        }
        if rand::random_range(0..=100) <= all_species[self.sid as usize].neurotic_rate {
            return Some(self.random_move());
        }
        let longer_range = if all_species[self.sid as usize].detect_positive_range
            > all_species[self.sid as usize].detect_negative_range
        {
            all_species[self.sid as usize].detect_positive_range
        } else {
            all_species[self.sid as usize].detect_negative_range
        };
        let Some(all_local_creatures) =
            self.scan_area_leader(map.width, map.height, sorted_creatures, longer_range)
        else {
            if let Some(random_step) = (self.pos.to_coor_isize()
                + CoorIsize::new(
                    rand::random_range(-1_i8..2_i8) as isize,
                    rand::random_range(-1_i8..2_i8) as isize,
                ))
            .and_then(|c| Some(c.to_coor_usize()))
            {
                return Some(ActorMovement::new(
                    self.pos,
                    crate::little_life_sim::Movement::Walk,
                    Some(random_step),
                ));
            } else {
                return None;
            }
        };
        let all_food = self.scan_local_food(&all_local_creatures, all_species);
        let all_eatale_animals = self.scan_local_eatable_animals(&all_local_creatures, all_species);
        let all_kinds = if self.mating_cold_down != 0
            && rand::random_range(0..100) >= all_species[self.sid as usize].neurotic_rate
        {
            None
        } else {
            self.scan_local_kind(&all_local_creatures, all_species)
        };
        let all_enemies = self.scan_local_threaten(&all_local_creatures, all_species);
        let mut good_weight: usize = 0;
        let mut bad_weight: usize = 0;
        if let Some(have_food) = &all_food {
            good_weight += have_food.len();
        }
        if let Some(have_eatable_animals) = &all_eatale_animals {
            good_weight += have_eatable_animals.len();
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
            a if a <= good_weight => {
                Some(self.get_movement_good_weight(all_food, all_kinds, all_eatale_animals, map))
            }
            a if a > good_weight => Some(self.get_movement_bad_weight(all_enemies)),
            _ => Some(self.random_move()),
        }
    }
    fn get_nearby_good_creature(
        &self,
        all_food: &Option<Vec<CoorUsize>>,
        all_kinds: &Option<Vec<CoorUsize>>,
        all_eatale_animals: &Option<Vec<CoorUsize>>,
        map: &gen_map::GameMap,
    ) -> Option<ActorMovement> {
        let go_eat = all_food.as_ref().and_then(|c| self.is_able_to_eat(&c, map));
        let go_mating = all_kinds
            .as_ref()
            .and_then(|c| self.is_able_to_mating(&c, map));
        let go_atk_food = all_eatale_animals
            .as_ref()
            .and_then(|c| self.is_able_to_atk(&c, map));
        let all_nearby: Vec<ActorMovement> = vec![go_eat, go_mating, go_atk_food]
            .iter()
            .filter_map(|&c| c)
            .collect();
        if all_nearby.len() != 0 {
            let random_movement: ActorMovement =
                all_nearby[rand::random_range(0..all_nearby.len())];
            Some(random_movement)
        } else {
            None
        }
    }
    fn get_movement_good_weight(
        &self,
        all_food: Option<Vec<CoorUsize>>,
        all_kinds: Option<Vec<CoorUsize>>,
        all_eatale_animals: Option<Vec<CoorUsize>>,
        map: &gen_map::GameMap,
    ) -> ActorMovement {
        let all_possible_creatures = vec![&all_food, &all_kinds, &all_eatale_animals];
        let all_creatures: Vec<&Vec<CoorUsize>> = all_possible_creatures
            .iter()
            .filter_map(|&c| c.as_ref())
            .collect();
        if all_creatures.len() == 0 {
            return self.random_move();
        }
        if let Some(nearby_movement) =
            self.get_nearby_good_creature(&all_food, &all_kinds, &all_eatale_animals, map)
        {
            return nearby_movement;
        }
        if let Some(new_coor) = all_creatures
            .iter()
            .filter_map(|&cs| {
                cs.iter()
                    .min_by_key(|&&c| self.pos.abs_usize(c))
                    .map(|c| self.pos.to_coor_isize().vector_to(c.to_coor_isize()))
            })
            .min_by_key(|&c| self.pos.abs_isize(c))
            .and_then(|closest_coor_dir| self.pos.to_coor_isize() + closest_coor_dir)
            .and_then(|coor| Some(coor.to_coor_usize()))
        {
            return ActorMovement::new(self.pos, Walk, Some(new_coor));
        }
        return self.random_move();
    }
    fn get_movement_bad_weight(&self, all_enemies: Option<Vec<CoorUsize>>) -> ActorMovement {
        let Some(enemies) = all_enemies else {
            return self.random_move();
        };
        let mut all_vector = CoorIsize::new(0, 0);
        for i in enemies.iter() {
            if let Some(new_coor) = i.vector_to(self.pos.to_coor_isize()) + all_vector {
                all_vector = new_coor
            }
        }
        let final_move = CoorIsize::new(
            ((all_vector.x) > (0)) as isize - ((all_vector.x) < (0)) as isize,
            ((all_vector.y) > (0)) as isize - ((all_vector.y) < (0)) as isize,
        );
        if final_move == CoorIsize::new(0, 0) {
            return self.random_move();
        } else {
            return ActorMovement::new(
                self.pos,
                Walk,
                Some(CoorUsize::new(
                    self.pos.x.saturating_add_signed(final_move.x),
                    self.pos.y.saturating_add_signed(final_move.y),
                )),
            );
        }
    }
}
