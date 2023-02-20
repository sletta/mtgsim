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

// const LAND_FLAG : u32           = 0x01;
// const CREATURE_FLAG : u32       = 0x02;
// const PLANESWALKER_FLAG : u32   = 0x04;
// const ARTIFACT_FLAG : u32       = 0x08;
// const ENCHANTMENT_FLAG : u32    = 0x10;
// const SORCERY_FLAG : u32        = 0x20;
// const INSTANT_FLAG : u32        = 0x40;

#[derive(Debug, Clone)]
pub struct Card {
    pub name: String,
    pub cmc: i32,
    pub mana_cost: mana::Pool,
    pub type_string: String,
    pub types: BitFlags<Types>,

    pub produced_mana: Option<mana::Mana>,
    pub enters_tapped : bool,
}

pub fn parse_types(types : &str) -> BitFlags<Types, u8> {
    let lower_cased = types.to_lowercase();
    let mut flags = BitFlags::empty();
    if types.find("land").is_some() {
        flags |= Types::Land;
    }
    if types.find("creature").is_some() {
        flags |= Types::Creature;
    }
    if types.find("planeswalker").is_some() {
        flags |= Types::Planeswalker;
    }
    if types.find("artifact").is_some() {
        flags |= Types::Artifact;
    }
    if types.find("enchantment").is_some() {
        flags |= Types::Enchantment;
    }
    if types.find("sorcery").is_some() {
        flags |= Types::Sorcery;
    }
    if types.find("instant").is_some() {
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
