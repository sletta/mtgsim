use crate::mana;
use enumflags2::{bitflags, BitFlags};

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Types {
    Land = 0x01,
    Creature = 0x02,
    Planeswalker = 0x04,
    Artifact = 0x08,
    Enchantment = 0x10,
    Sorcery = 0x20,
    Instant = 0x40,
}

#[derive(Debug, Clone)]
// pub enum TapEffect {
//     None,
//     ProduceMana(mana::Pool), // Lands, Mana Rocks and Mana Dorks
// }

/*

{
    "name": "Commander's Sphere",
    "tap": {
        "effect": "produce-mana",
        "produced": "{B/G/R/U/W}"
    },
    "sacrifice:" {
        "effect": "draw",
        "count": 1
    }
},

{
    "name": "Evolving Wilds",
    "sacrifice": {
        "effect": "land-to-battelfield",
        "types": "basic land"
    }
}

{
    "name": "Cultivate",
    "play": [
        {
            "effect": "land-to-hand",
            "types": "basic land"
        }, {
            "effect": "land-to-battlefield",
            "types": "basic land"
        } ]
},

{
    "name": "Elemental Bond",
    "upkeep": {
        "effect": "draw",
        "count": [0, 0, 1, 1, 1, 1, 2, 2, 3, 4]
    }
}

{
    "name": "War Room",
    "activate": {
        "effect": "draw",
        "cost": "{C}{C}{C}",
        "count:": 1
    }
}


 */

pub enum Effect {
    None,
    ProduceMana(mana::Pool), // like 'Dark Ritual'
    LandToHand(Vec<String>), // like 'Borderland Ranger'
    LandToBattlefield(Vec<String>), // 'Rampant Growth' or 'Cultivate'
    Draw(Vec<u32>),                 // like 'Harmonize' or 'Read the Bones'
    Multiple(Vec<Effect>),
}

#[derive(Debug)]
pub struct CardData {
    pub name: String,
    pub cmc: i32,
    pub mana_cost: Option<mana::Pool>,
    pub type_string: String,
    pub types: BitFlags<Types>,

    pub produced_mana: Option<mana::Mana>,
    pub enters_tapped : bool,

    pub on_tap : Effect,
    pub on_play : Effect,
    pub on_activate : Effect,
    pub on_sac : Effect,
    pub on_upkeep : Effect,
}

#[derive(Debug, Clone)]
pub struct Card<'db> {
    pub id : u32,
    pub data : &'db CardData,
    pub tapped: bool
}

pub fn parse_types(types : &str) -> BitFlags<Types, u8> {
    let lower_cased = types.to_lowercase();
    let mut flags = BitFlags::empty();
    if lower_cased.find("land").is_some() {
        flags |= Types::Land;
    }
    if lower_cased.find("creature").is_some() {
        flags |= Types::Creature;
    }
    if lower_cased.find("planeswalker").is_some() {
        flags |= Types::Planeswalker;
    }
    if lower_cased.find("artifact").is_some() {
        flags |= Types::Artifact;
    }
    if lower_cased.find("enchantment").is_some() {
        flags |= Types::Enchantment;
    }
    if lower_cased.find("sorcery").is_some() {
        flags |= Types::Sorcery;
    }
    if lower_cased.find("instant").is_some() {
        flags |= Types::Instant;
    }
    return flags;
}

impl<'db> Card<'db> {
    pub fn new(data : &'db CardData) -> Self {
        let card = Card {
            id: 0,
            data: data,
            tapped: false,
        };
        return card;
    }

    pub fn new_with_id(the_id : u32, data : &'db CardData) -> Self {
        let mut card = Card::new(data);
        card.id = the_id;
        return card;
    }

    pub fn is_type(&self, t : Types) -> bool {
        return self.data.types.contains(t);
    }
}

impl<'db> std::fmt::Display for Card<'db> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - [{}] #{}", self.data.name, self.data.types, self.id)?;
        if self.tapped {
            write!(f, " *TAPPED*")?;
        }
        if self.data.mana_cost.is_some() {
            write!(f, " - {} ({})", self.data.mana_cost.as_ref().unwrap(), self.data.cmc)?;
        }
        return Ok(());
    }
}

impl CardData {
    #[cfg(test)]
    pub fn make_sol_ring_data() -> CardData {
        return CardData {
            name: "Sol Ring".to_string(),
            cmc: 1,
            mana_cost: Some(mana::Pool { sequence: vec![mana::COLORLESS] }),
            type_string: "Artifact".to_string(),
            types: enumflags2::make_bitflags!(Types::{Artifact}),
            produced_mana: Some(mana::COLORLESS),
            enters_tapped: false,
            on_tap: Effect::ProduceMana(mana::Pool { sequence: vec![mana::COLORLESS, mana::COLORLESS] }),
            on_play: Effect::None,
            on_activate: Effect::None,
            on_sac: Effect::None,
            on_upkeep: Effect::None,
        };
    }

    #[cfg(test)]
    pub fn make_commanders_sphere_data() -> CardData {
        return CardData {
            name: "Commander's Sphere".to_string(),
            cmc: 1,
            mana_cost: Some(mana::Pool { sequence: vec![mana::COLORLESS, mana::COLORLESS, mana::COLORLESS] }),
            type_string: "Artifact".to_string(),
            types: enumflags2::make_bitflags!(Types::{Artifact}),
            produced_mana: Some(mana::ALL),
            enters_tapped: false,
            on_tap: Effect::ProduceMana(mana::Pool { sequence: vec![mana::ALL] }),
            on_play: Effect::None,
            on_activate: Effect::None,
            on_sac: Effect::Draw(vec![1]),
            on_upkeep: Effect::None,
        };
    }

    #[cfg(test)]
    pub fn make_plains_data() -> CardData {
        return CardData {
            name: "Plains".to_string(),
            cmc: 0,
            mana_cost: None,
            type_string: "Basic Land".to_string(),
            types: enumflags2::make_bitflags!(Types::{Land}),
            produced_mana: Some(mana::WHITE),
            enters_tapped: false,
            on_tap: Effect::ProduceMana(mana::Pool { sequence: vec![mana::WHITE] }),
            on_play: Effect::None,
            on_activate: Effect::None,
            on_sac: Effect::None,
            on_upkeep: Effect::None,
        };
    }

    #[cfg(test)]
    pub fn make_swamp_data() -> CardData {
        return CardData {
            name: "Swamp".to_string(),
            cmc: 0,
            mana_cost: None,
            type_string: "Basic Land".to_string(),
            types: enumflags2::make_bitflags!(Types::{Land}),
            produced_mana: Some(mana::BLACK),
            enters_tapped: false,
            on_tap: Effect::ProduceMana(mana::Pool { sequence: vec![mana::BLACK] }),
            on_play: Effect::None,
            on_activate: Effect::None,
            on_sac: Effect::None,
            on_upkeep: Effect::None,
        };
    }

    #[cfg(test)]
    pub fn make_command_tower_data() -> CardData {
        return CardData {
            name: "Command Tower".to_string(),
            cmc: 0,
            mana_cost: None,
            type_string: "Land".to_string(),
            types: enumflags2::make_bitflags!(Types::{Land}),
            produced_mana: Some(mana::ALL),
            enters_tapped: false,
            on_tap: Effect::ProduceMana(mana::Pool { sequence: vec![mana::ALL] }),
            on_play: Effect::None,
            on_activate: Effect::None,
            on_sac: Effect::None,
            on_upkeep: Effect::None,
        };
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_carddata_parse_types() {
        assert_eq!(parse_types("something land something"), enumflags2::make_bitflags!(Types::{ Land }));
        assert_eq!(parse_types("something creature something"), enumflags2::make_bitflags!(Types::{ Creature }));
        assert_eq!(parse_types("something artifact something"), enumflags2::make_bitflags!(Types::{ Artifact }));
        assert_eq!(parse_types("something planeswalker something"), enumflags2::make_bitflags!(Types::{ Planeswalker }));
        assert_eq!(parse_types("something enchantment something"), enumflags2::make_bitflags!(Types::{ Enchantment }));
        assert_eq!(parse_types("something sorcery something"), enumflags2::make_bitflags!(Types::{ Sorcery }));
        assert_eq!(parse_types("something instant something"), enumflags2::make_bitflags!(Types::{ Instant }));
        assert_eq!(parse_types("land creature - angel"), enumflags2::make_bitflags!(Types::{ Land | Creature }));
        assert_eq!(parse_types("legendary artifact - equipment"), enumflags2::make_bitflags!(Types::{ Artifact }));
        assert_eq!(parse_types("something land something"), enumflags2::make_bitflags!(Types::{ Land }));
        assert_eq!(parse_types("legendary enchantment creature"), enumflags2::make_bitflags!(Types::{ Creature | Enchantment }));
    }
}
