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


{   "name": "Expand the Sphere"
{   "name": "Path of Discovery"
{   "name": "Pyramid of the Pantheon"
{   "name": "Scale the Heights"

{   "name": "Scale the Heights",
    "abilities": [
        { "trigger": "cast", "effect": { "type": "land-limit", "count": 1 } },
        { "trigger": "cast", "effect": { "type": "draw", "count": 1 } }
    ]
},


 */

#[derive(Debug, PartialEq)]
pub enum Effect {
    ProduceMana(ManaPool),
    FetchLand { to_hand: Vec<String>, to_battlefield: Vec<String> }, // like 'Cultivate'
    LandLimit(u32), // the increase in playable lands
    Draw(Vec<u32>),                 // like 'Harmonize' or 'Read the Bones'
}

#[derive(Debug, PartialEq)]
pub enum Trigger {
    Cast,
    Activated,
    Upkeep
}

#[derive(Debug, PartialEq)]
pub enum Cost {
    None,
    Tap,
    Sacrifice,
    Mana(ManaPool),
    TapSacrifice,
    TapMana(ManaPool),
    TapManaSacrifice(ManaPool),
}

#[derive(Debug)]
pub struct Ability {
    pub trigger: Trigger,
    pub cost: Cost,
    pub effect: Effect,
    pub availability: f32,
}

#[derive(Debug)]
pub struct CardData {
    pub name: String,
    pub cmc: u32,
    pub mana_cost: Option<ManaPool>,
    pub type_string: String,
    pub types: BitFlags<Types>,

    pub produced_mana: Option<Mana>,
    pub enters_tapped: bool,

    pub abilities: Option<Vec<Ability>>,
}

#[derive(Debug, Clone)]
pub struct Card<'db> {
    pub id: u32,
    pub data: &'db CardData,
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

    pub fn produced_mana(&self) -> Option<ManaPool> {
        for ability in self.data.abilities.iter().flatten() {
            match &ability.effect {
                Effect::ProduceMana(mana) => return Some(mana.clone()),
                _ => ()
            }
        }
        return None;
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

impl std::fmt::Display for Cost {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Cost::None => write!(f, "none"),
            Cost::Tap => write!(f, "tap"),
            Cost::Sacrifice => write!(f, "sac"),
            Cost::Mana(pool) => write!(f, "{}", pool),
            Cost::TapSacrifice => write!(f, "tap,sac"),
            Cost::TapMana(pool) => write!(f, "tap,{}", pool),
            Cost::TapManaSacrifice(pool) => write!(f, "tap,sac,{}", pool),
        }
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Trigger::Cast => write!(f, "cast"),
            Trigger::Activated => write!(f, "activated"),
            Trigger::Upkeep => write!(f, "upkeep")
        }
    }
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Effect::ProduceMana(pool) => write!(f, "produce={}", pool),
            Effect::FetchLand { to_hand: hand, to_battlefield: bf } => write!(f, "fetch={}/{}", hand.len(), bf.len()),
            Effect::LandLimit(increase) => write!(f, "land-limit=+{}", increase),
            Effect::Draw(ratios) => write!(f, "draw({:?})", ratios)
        }
    }
}

impl std::fmt::Display for Ability {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Ability({} {} {})", self.effect, self.trigger, self.cost)
    }
}

impl CardData {

    #[cfg(test)]
    pub fn make_sol_ring_data() -> CardData {
        return CardData {
            name: "Sol Ring".to_string(),
            cmc: 1,
            mana_cost: Some(ManaPool::new_from_sequence(&vec![COLORLESS])),
            type_string: "Artifact".to_string(),
            types: enumflags2::make_bitflags!(Types::{Artifact}),
            produced_mana: Some(COLORLESS),
            enters_tapped: false,
            abilities: Some(vec![ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(ManaPool::new_from_sequence(&vec![COLORLESS, COLORLESS])),
                availability: 1.0
            }])
        };
    }

    #[cfg(test)]
    pub fn make_commanders_sphere_data() -> CardData {
        return CardData {
            name: "Commander's Sphere".to_string(),
            cmc: 1,
            mana_cost: Some(ManaPool::new_from_sequence(&vec![COLORLESS, COLORLESS, COLORLESS])),
            type_string: "Artifact".to_string(),
            types: enumflags2::make_bitflags!(Types::{Artifact}),
            produced_mana: Some(ALL),
            enters_tapped: false,
            abilities: Some(vec! [ Ability {
                trigger: Trigger::Activated,
                cost: Cost::Tap,
                effect: Effect::ProduceMana(ManaPool::new_from_sequence(&vec![ALL])),
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
                effect: Effect::ProduceMana(ManaPool::new_from_sequence(&vec![WHITE])),
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
                effect: Effect::ProduceMana(ManaPool::new_from_sequence(&vec![BLACK])),
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
                effect: Effect::ProduceMana(ManaPool::new_from_sequence(&vec![ALL])),
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
                effect: Effect::ProduceMana(ManaPool::new_from_sequence(&vec![Mana::make_dual(Color::Black, Color::Green)])),
                availability: 1.0
            }])
        };
    }

    #[cfg(test)]
    pub fn make_elk_data() -> CardData {
        return CardData {
            name: "Just an Elk".to_string(),
            cmc: 3,
            mana_cost: Some(ManaPool::new_from_sequence(&vec![COLORLESS, COLORLESS, GREEN])),
            type_string: "Creature".to_string(),
            types: enumflags2::make_bitflags!(Types::{Creature}),
            produced_mana: None,
            enters_tapped: false,
            abilities: None
        };
    }
}

impl Cost {
    pub fn is_none(&self) -> bool {
        match self {
            Cost::None => true,
            _ => false
        }
    }
    pub fn is_tap(&self) -> bool {
        match self {
            Cost::Tap => true,
            Cost::TapMana(_) => true,
            Cost::TapManaSacrifice(_) => true,
            _ => false
        }
    }
    pub fn is_mana(&self) -> Option<&ManaPool> {
        match self {
            Cost::Mana(pool) => Some(pool),
            Cost::TapMana(pool) => Some(pool),
            Cost::TapManaSacrifice(pool) => Some(pool),
            _ => None
        }
    }
    pub fn is_sacrifice(&self) -> bool {
        match self {
            Cost::Sacrifice => true,
            Cost::TapSacrifice => true,
            Cost::TapManaSacrifice(_) => true,
            _ => false
        }
    }
}

impl Trigger {
    pub fn is_cast(&self) -> bool {
        match self {
            Trigger::Cast => true,
            _ => false
        }
    }
    pub fn is_activated(&self) -> bool {
        match self {
            Trigger::Activated => true,
            _ => false
        }
    }
    pub fn is_upkeep(&self) -> bool {
        match self {
            Trigger::Upkeep => true,
            _ => false
        }
    }
}

impl Effect {
    pub fn is_produce_mana(&self) -> bool {
        match self {
            Effect::ProduceMana(_) => true,
            _ => false
        }
    }
    pub fn is_draw(&self) -> bool {
        match self {
            Effect::Draw(_) => true,
            _ => false
        }
    }
    pub fn is_fetch_land(&self) -> bool {
        match self {
            Effect::FetchLand{to_hand: _, to_battlefield: _} => true,
            _ => false
        }
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
