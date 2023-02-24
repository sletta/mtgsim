use crate::mana;
use enumflags2::{bitflags, make_bitflags, BitFlags};

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
pub enum Ramp {
    ManaProducer(mana::Pool),
    LandFetch(Vec<String>)
}

#[derive(Debug, Clone)]
pub enum Draw {
    OneShot(Vec<i32>),
    PerTurn(Vec<i32>),
}

#[derive(Debug, Clone)]
pub struct Card {
    pub name: String,
    pub cmc: i32,
    pub mana_cost: Option<mana::Pool>,
    pub type_string: String,
    pub types: BitFlags<Types>,

    pub produced_mana: Option<mana::Mana>,
    pub enters_tapped : bool,

    pub ramp : Option<Ramp>,
    pub draw : Option<Draw>,
    pub land_mana : Option<mana::Pool>
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

impl Card {
    pub fn is_land(&self) -> bool {
        return self.types.contains(Types::Land);
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_card_parse_types() {
        assert_eq!(parse_types("something land something"), make_bitflags!(Types::{ Land }));
        assert_eq!(parse_types("something creature something"), make_bitflags!(Types::{ Creature }));
        assert_eq!(parse_types("something artifact something"), make_bitflags!(Types::{ Artifact }));
        assert_eq!(parse_types("something planeswalker something"), make_bitflags!(Types::{ Planeswalker }));
        assert_eq!(parse_types("something enchantment something"), make_bitflags!(Types::{ Enchantment }));
        assert_eq!(parse_types("something sorcery something"), make_bitflags!(Types::{ Sorcery }));
        assert_eq!(parse_types("something instant something"), make_bitflags!(Types::{ Instant }));
        assert_eq!(parse_types("land creature - angel"), make_bitflags!(Types::{ Land | Creature }));
        assert_eq!(parse_types("legendary artifact - equipment"), make_bitflags!(Types::{ Artifact }));
        assert_eq!(parse_types("something land something"), make_bitflags!(Types::{ Land }));
        assert_eq!(parse_types("legendary enchantment creature"), make_bitflags!(Types::{ Creature | Enchantment }));
    }
}
