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
pub enum Ramp {
    ManaProducer(mana::Pool),
    LandToBattlefield(Vec<String>)
}

#[derive(Debug, Clone)]
pub enum Draw {
    OneShot(Vec<i32>),
    PerTurn(Vec<i32>),
    Activated(Vec<i32>, mana::Pool),
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

    pub ramp : Option<Ramp>,
    pub draw : Option<Draw>,
    pub land_mana : Option<mana::Pool>
}

#[derive(Debug, Clone)]
pub struct Card<'a> {
    pub data : &'a CardData,
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

impl CardData {
}

impl<'a> Card<'a> {
    pub fn new(data : &'a CardData) -> Self {
        return Card {
            data: data,
            tapped: false,
        }
    }
}

impl<'a> std::fmt::Display for Card<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - [{}]", self.data.name, self.data.types)?;
        if self.tapped {
            write!(f, " *TAPPED*")?;
        }
        if self.data.mana_cost.is_some() {
            write!(f, " - {} ({})", self.data.mana_cost.as_ref().unwrap(), self.data.cmc)?;
        }
        return Ok(());
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
