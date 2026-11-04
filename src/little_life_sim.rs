use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

use crate::game_tools::CoorUsize;
use crate::game_tools::{self, CoorIsize};
use crate::gen_map;
use std::collections::HashMap;
pub struct IdGenerator {
    curr_id: u8,
}
impl IdGenerator {
    pub fn new() -> Self {
        Self { curr_id: 0 }
    }
    pub fn gen_id(&mut self) -> u8 {
        let id = self.curr_id;
        self.curr_id += 1;
        id
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
        id_generator: &mut IdGenerator,
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
    pub hp_left: usize,
    pub speed: usize,
}
impl Creature {
    pub fn new(
        specie: &Specie,
        area: &mut gen_map::GameMap,
        pos: game_tools::CoorUsize,
        gender: Gender,
    ) -> Result<Self, ()> {
        if area.get_str_map_val(&pos) != " " {
            return Err(());
        }
        area.change_str_map_val(&pos, &specie.icon.to_string());
        let random_debuff = |original_value: usize| -> usize {
            let temp = original_value - rand::random_range(0..=original_value);
            if temp == 0 && rand::random_range(0..=100) < specie.trophic_level.get_die_rate() {
                0
            } else if temp != 0 {
                temp
            } else {
                original_value - rand::random_range(0..((original_value / 2) + 1))
            }
        };
        let hp_left = random_debuff(specie.max_hp);
        let speed = (specie.max_speed as f32 * rand::random_range(0.0..=1.0)) as usize;
        let energy_left = random_debuff(specie.max_energy);
        let mating_cold_down = if specie.max_mating_cold_down < 1 {
            0
        } else {
            specie.max_mating_cold_down
                - rand::random_range(
                    (specie.max_mating_cold_down / 2)..=specie.max_mating_cold_down,
                )
        };
        let sid = specie.get_sid();
        Ok(Creature {
            sid,
            pos,
            gender,
            mating_cold_down,
            energy_left,
            hp_left,
            speed,
        })
    }
    pub fn get_hurt(&mut self, damage: usize) {
        if !(damage >= self.hp_left) {
            self.hp_left -= damage;
            return;
        }
        self.hp_left = 0;
    }
    pub fn is_dead(&self) -> bool {
        self.hp_left == 0
    }

    pub fn is_target_in_range(&self, target_coor: CoorUsize, range: usize) -> bool {
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
        all_creature_place: &[&'a Creature],
        range: usize,
    ) -> Option<Vec<&'a Creature>> {
        let mut sort_creatures: Vec<&Creature> = all_creature_place.iter().copied().collect();
        sort_creatures.par_sort_unstable_by_key(|c| c.pos);
        let index_sort_creatures: HashMap<CoorUsize, usize> = sort_creatures
            .iter()
            .enumerate()
            .map(|(i, &c)| (c.pos, i))
            .collect();
        let self_index = index_sort_creatures.get(&self.pos).unwrap();
        let range_1d = (range * map_width + range) / 2;
        let left_side = (self_index.checked_sub(range_1d)).and_then(|c| sort_creatures.get(c));
        let right_side = (self_index.checked_add(range_1d)).and_then(|c| sort_creatures.get(c));
        match (left_side, right_side) {
            (Some(a), Some(b)) => {
                game_tools::debug("a b", true);
                let is_left_too_tight = (a.pos.to_coor_isize() * -1)
                    .and_then(|c| c + self.pos.to_coor_isize())
                    .and_then(|c| Some(c.x < range as isize && c.y < range as isize));
                let is_right_too_tight = (b.pos.to_coor_isize() + self.pos.to_coor_isize())
                    .and_then(|c| Some(c.x < range as isize && c.y < range as isize));

                if is_left_too_tight.is_some_and(|c| c) && is_right_too_tight.is_some_and(|c| c) {
                    return self.scan_area_when_dense(
                        map_width,
                        map_height,
                        all_creature_place,
                        range,
                    );
                }
                self.scan_area_when_sparse(map_width, &sort_creatures, &index_sort_creatures, range)
            }
            (Some(a), None) => {
                game_tools::debug("a", true);
                let is_left_too_tight = (a.pos.to_coor_isize() * -1)
                    .and_then(|c| c + self.pos.to_coor_isize())
                    .and_then(|c| Some(c.x < range as isize && c.y < range as isize));
                if is_left_too_tight.is_some_and(|c| c) {
                    return self.scan_area_when_dense(
                        map_width,
                        map_height,
                        all_creature_place,
                        range,
                    );
                }
                self.scan_area_when_sparse(map_width, &sort_creatures, &index_sort_creatures, range)
            }
            (None, Some(b)) => {
                game_tools::debug("b", true);
                let is_right_too_tight = (b.pos.to_coor_isize() + self.pos.to_coor_isize())
                    .and_then(|c| Some(c.x < range as isize && c.y < range as isize));

                if is_right_too_tight.is_some_and(|c| c) {
                    return self.scan_area_when_dense(
                        map_width,
                        map_height,
                        all_creature_place,
                        range,
                    );
                }
                self.scan_area_when_sparse(map_width, &sort_creatures, &index_sort_creatures, range)
            }
            _ => {
                game_tools::debug("Na", true);
                self.scan_area_when_sparse(map_width, &sort_creatures, &index_sort_creatures, range)
            }
        }
    }
    fn scan_area_when_sparse<'a>(
        &self,
        map_width: usize,
        sort_creatures: &Vec<&'a Creature>,
        index_sort_creatures: &HashMap<CoorUsize, usize>,
        range: usize,
    ) -> Option<Vec<&'a Creature>> {
        panic!()
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

    pub fn scan_step_where(
        &self,
        all_creature_place: &[&Creature],
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
        let Some(all_local_creatures) =
            self.scan_area_when_dense(map_width, map_height, all_creature_place, longer_range)
        else {
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
}
fn main(
    weigth: usize,
    heigth: usize,
    inital_peaceful_life_number: usize,
    peaceful_life_dietary_categorie_generate_time: f32,
    peaceful_life_dietary_categorie_generate_percentage: f32,
    inital_hostile_life_number: usize,
) {
}
