use crate::game_tools;
use crate::game_tools::CoorUsize;
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
#[derive(Hash, PartialEq, PartialOrd, Eq)]
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
#[derive(Clone, Copy)]
pub struct Specie {
    sid: u8,
    pub max_hp: usize,
    pub max_speed: usize,
    pub max_energy: usize,
    pub trophic_level: TrophicLevel,
    pub dietarycategories: DietaryCategories,
    pub icon: char,
    pub detect_positive_range: usize,
    pub detect_negative_range: usize,
}
impl Specie {
    pub fn new(
        max_hp: usize,
        max_speed: usize,
        max_energy: usize,
        trophic_level: TrophicLevel,
        dietarycategories: DietaryCategories,
        icon: char,
        detect_positive_range: usize,
        detect_negative_range: usize,
        id_generator: &mut IdGenerator,
    ) -> Self {
        Specie {
            sid: id_generator.gen_id(),
            max_hp,
            max_speed,
            max_energy,
            trophic_level,
            dietarycategories,
            icon,
            detect_positive_range,
            detect_negative_range,
        }
    }
}
#[derive(Hash, PartialEq, PartialOrd, Eq)]
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
        mating_cold_down: usize,
    ) -> Result<Self, ()> {
        if area.get_str_map_val(&pos) != " " {
            return Err(());
        }
        area.change_str_map_val(&pos, &specie.icon.to_string());
        let hp_left = specie.max_hp;
        let speed = specie.max_speed;
        let energy_left = specie.max_energy;
        let sid = specie.sid;
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
    fn check_vertical_range_stuffs<'a>(
        map: &gen_map::GameMap,
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
                .filter(|x| !(*x > map.width - 1))
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
    pub fn get_near_by_stuffs_place<'a>(
        &self,
        map: &gen_map::GameMap,
        all_creature_place: &[&'a Creature],
        range: usize,
    ) -> Option<Vec<&'a Creature>> {
        let mut all_target_place: Vec<&Creature> = vec![];
        let all_creature_corr_and_creature: HashMap<CoorUsize, &Creature> =
            all_creature_place.iter().map(|c| (c.pos, *c)).collect();
        for y in 1..=range {
            if let Some(top_y) = self.pos.y.checked_sub_signed(y as isize) {
                let new_top_y_coor = CoorUsize::new(self.pos.x, top_y);
                if let Some(&k) = all_creature_corr_and_creature.get(&new_top_y_coor) {
                    all_target_place.push(k);
                };

                if let Some(new_coor) = Self::check_vertical_range_stuffs(
                    map,
                    &new_top_y_coor,
                    range,
                    &all_creature_corr_and_creature,
                ) {
                    all_target_place.extend(new_coor);
                }
            }

            if let Some(bottom_y) = self
                .pos
                .y
                .checked_add_signed(y as isize)
                .filter(|y| !(*y > map.height - 1))
            {
                let new_bottom_y_coor = CoorUsize::new(self.pos.x, bottom_y);
                if let Some(&k) = all_creature_corr_and_creature.get(&new_bottom_y_coor) {
                    all_target_place.push(k);
                }
                if let Some(new_coor) = Self::check_vertical_range_stuffs(
                    map,
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
    pub fn get_near_by_food_coor(
        &self,
        map: &gen_map::GameMap,
        all_creatures: &[&Creature],
        all_species: &[&Specie],
    ) -> Option<Vec<CoorUsize>> {
        let self_specie = &all_species[self.sid as usize];
        let all_food_coor: Vec<CoorUsize> = match self_specie {
            a if a.dietarycategories == DietaryCategories::Herbivorous => all_creatures
                .iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    target_specie.trophic_level == TrophicLevel::Producer
                })
                .map(|c| c.pos)
                .collect(),
            a if a.dietarycategories == DietaryCategories::Carnivorous => all_creatures
                .iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    target_specie.trophic_level != TrophicLevel::Producer
                        && self.sid != c.sid
                        && (self_specie.trophic_level == TrophicLevel::Top
                            || target_specie.trophic_level < self_specie.trophic_level)
                })
                .map(|c| c.pos)
                .collect(),
            a if a.dietarycategories == DietaryCategories::Omnivorous => all_creatures
                .iter()
                .filter(|c| {
                    let target_specie = &all_species[c.sid as usize];
                    self.sid != c.sid
                        && (self_specie.trophic_level == TrophicLevel::Top
                            || target_specie.trophic_level < self_specie.trophic_level)
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
    // pub fn get_near_by_negative_stuffs_coor(
    //     &self,
    //     map: &gen_map::GameMap,
    //     all_species: &[Creature],
    // ) -> Option<Vec<CoorUsize>> {
    //     let mut all_good_species_index: Vec<usize> = Vec::new();
    //     for s in 0..all_species.len() {
    //         if !(all_species[s].species.trophic_level == TrophicLevel::Top
    //             || all_species[s].species.trophic_level > self.specie.trophic_level)
    //         {
    //             continue;
    //         }
    //         all_good_species_index.push(s);
    //     }
    //     let mut all_good_pos: Vec<CoorUsize> = vec![];
    //     for gs in all_good_species_index {
    //         let Some(result) = self.get_near_by_stuffs_place(
    //             map,
    //             &all_species[gs]
    //                 .creature_data
    //                 .iter()
    //                 .map(|c| c.pos)
    //                 .collect(),
    //             self.specie.detect_positive_range,
    //         ) else {
    //             return None;
    //         };
    //         all_good_pos.extend(result);
    //     }
    //     Some(all_good_pos)
    // }

    // pub fn walk(&self, map: &gen_map::GameMap){
    //     is_near_negative = if !(self.dietary_categorie == DietaryCategories::Carnivorous)
    //     &&
    // }
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
