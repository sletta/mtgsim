use crate::mana;

#[derive(Debug, Clone)]
pub struct Card {
    pub name: String,
    pub cmc: i32,
    pub mana_cost: mana::Pool,
    pub types: String,

    pub produced_mana: Option<mana::Mana>,
    pub enters_tapped : bool,

}
