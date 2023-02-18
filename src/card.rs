use crate::mana;

#[derive(Debug, Clone)]
pub struct Card {
    pub name: String,
    pub cmc: i32,
    pub mana_cost: mana::Cost
}
