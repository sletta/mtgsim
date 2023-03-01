use crate::mana::*;
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

/*

{
    "name": "Commander's Sphere",
    "abilities": [
        {
            "trigger": "activated",
            "effect": { "type": "mana", "produces": "{B/G/R/W/U}" },
            "cost": "tap"
        }, {
            "trigger": "activated",
            "effect": { "type": "draw", "count": "1" },
            "cost": "sacrifice",
        }
    ]
},

{
    "name": "Evolving Wilds",
    "trigger": "activated",
    "cost": "tap",
    "effect": { "type": "fetch-land", "to-battlefield": "basic land" }
}

{
    "name": "Farseek",
    "trigger": "cast",
    "effect": { "type": "fetch-land", "to-battlefield": "mountain/island/plains/swamp" }
},

{
    "name": "Cultivate",
    "trigger": "cast",
    "effect": { "type": "fetch-land", "to-battlefield": "basic land", "to-hand": "basic land" }
},

{
    "name": "Elemental Bond",
    "trigger": "upkeep"
    "effect": { "type": "draw", "count": [0, 0, 1, 1, 1, 1, 2, 2, 3, 4] },
}

{
    "name": "War Room",
    "abilities": [
        {
            "trigger": "activated",
            "effect": { "type": "mana", "produces": "{C}" },
            "cost": "tap"
        }, {
            "trigger": "activated",
            "effect": { "type": "draw", "count": "1" },
            "cost": { "type": "tap-and-mana", "mana": {C}{C}{C}" },
            "availability": 0.5
        }
        ]
    }
}

{
    "name": "Black Market",
    "trigger": "upkeep",
    "availability": 0.5
    "effect": {
        "type": "mana",
        "produce": "{B}{B}{B}{B}{B}{B}{B}{B}",
    }
},

{
    "name": "Blighted Woodland",
    "abilities": [
        {
            "trigger": "activated",
            "effect": { "type": "mana", "produces": "{C}" },
            "cost": "tap"
        }, {
            "trigger": "activated",
            "effect": { "type": "land-fetch", "to-battlefield": [ "basic land", "basic land" ] },
            "cost": "tap-and-mana"
            "mana-cost": "{3}{G}"
        } ]


{   "name": "Cultivate"
{   "name": "Expand the Sphere"
{   "name": "Far Wanderings"
{   "name": "Myriad Landscape"
{   "name": "Path of Discovery"
{   "name": "Pyramid of the Pantheon"
{   "name": "Scale the Heights"
{   "name": "Vastwood Surge"

 */

#[derive(Debug)]
pub enum Effect {
    None,
    ProduceMana(Pool), // like 'Dark Ritual'
    FetchLand { to_hand : Vec<String>, to_battlefield: Vec<String> }, // like 'Cultivate'
    Draw(Vec<u32>),                 // like 'Harmonize' or 'Read the Bones'
}

#[derive(Debug)]
pub enum Trigger {
    Cast,
    Activated,
    Upkeep
}

#[derive(Debug)]
pub enum Cost {
    None,
    Tap,
    Mana(Pool),
    Sacrifice,
    TapAndMana(Pool),
    TapSacrificeMana(Pool)
}

#[derive(Debug)]
pub struct Ability {
    pub trigger: Trigger,
    pub cost : Cost,
    pub effect : Effect,
    pub availability : f32,
}

#[derive(Debug)]
pub struct CardData {
    pub name: String,
    pub cmc: u32,
    pub mana_cost: Option<Pool>,
    pub type_string: String,
    pub types: BitFlags<Types>,

    pub produced_mana: Option<Mana>,
    pub enters_tapped : bool,

    pub abilities : Option<Vec<Ability>>,
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

    #[cfg(test)]
    pub fn new_with_id(the_id : u32, data : &'db CardData) -> Self {
        let mut card = Card::new(data);
        card.id = the_id;
        return card;
    }

    pub fn is_type(&self, t : Types) -> bool {
        return self.data.types.contains(t);
    }

    pub fn is_ramp(&self) -> bool {
        match &self.data.abilities {
            Some(abilities) => for ability in abilities {
                match &ability.effect {
                    Effect::ProduceMana(_) => {
                        if self.is_type(Types::Land) {
                            continue;
                        } else {
                            return true;
                        }
                    },
                    Effect::FetchLand{to_hand: _, to_battlefield: _} => return true,
                    _ => continue,
                }
            },
            None => (),
        }
        return false;
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

    pub fn calculate_produced_mana(&self) -> Option<Mana> {
        for ability in self.abilities.as_ref()?.iter() {
            match &ability.effect {
                Effect::ProduceMana(pool) => return Some(pool.union_of_all_colors()),
                _ => ()
            }
        }
        return None;
    }

    #[cfg(test)]
    pub fn make_sol_ring_data() -> CardData {
        return CardData {
            name: "Sol Ring".to_string(),
            cmc: 1,
            mana_cost: Some(Pool { sequence: vec![COLORLESS] }),
            type_string: "Artifact".to_string(),
            types: enumflags2::make_bitflags!(Types::{Artifact}),
            produced_mana: Some(COLORLESS),
            enters_tapped: false,
            abilities: Some(vec![ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(make_pool![COLORLESS, COLORLESS]),
                availability: 1.0
            }])
        };
    }

    #[cfg(test)]
    pub fn make_commanders_sphere_data() -> CardData {
        return CardData {
            name: "Commander's Sphere".to_string(),
            cmc: 1,
            mana_cost: Some(Pool { sequence: vec![COLORLESS, COLORLESS, COLORLESS] }),
            type_string: "Artifact".to_string(),
            types: enumflags2::make_bitflags!(Types::{Artifact}),
            produced_mana: Some(ALL),
            enters_tapped: false,
            abilities: Some(vec! [ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(make_pool![ALL]),
                availability: 1.0
            }])
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
            produced_mana: Some(WHITE),
            enters_tapped: false,
            abilities: Some(vec! [ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(make_pool![WHITE]),
                availability: 1.0
            }])
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
            produced_mana: Some(BLACK),
            enters_tapped: false,
            abilities: Some(vec! [ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(make_pool![BLACK]),
                availability: 1.0
            }])
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
            produced_mana: Some(ALL),
            enters_tapped: false,
            abilities: Some(vec! [ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(make_pool![ALL]),
                availability: 1.0
            }])
        };
    }

    #[cfg(test)]
    pub fn make_jungle_hollow_data() -> CardData {
        return CardData {
            name: "Jungle Hollow".to_string(),
            cmc: 0,
            mana_cost: None,
            type_string: "Land".to_string(),
            types: enumflags2::make_bitflags!(Types::{Land}),
            produced_mana: Some(Mana::make_dual(Color::Black, Color::Green)),
            enters_tapped: true,
            abilities: Some(vec![ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(make_pool![Mana::make_dual(Color::Black, Color::Green)]),
                availability: 1.0
            }])
        };
    }

    #[cfg(test)]
    pub fn make_elk_data() -> CardData {
        return CardData {
            name: "Just an Elk".to_string(),
            cmc: 3,
            mana_cost: Some(make_pool![COLORLESS, COLORLESS, GREEN]),
            type_string: "Creature".to_string(),
            types: enumflags2::make_bitflags!(Types::{Creature}),
            produced_mana: None,
            enters_tapped: false,
            abilities: None
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
