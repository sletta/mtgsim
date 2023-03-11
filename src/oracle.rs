use crate::mana;
use crate::card;
use lazy_static::lazy_static;

use regex::Regex;

pub struct Context<'a> {
    text: &'a str,
    card_name: &'a str
}

pub fn parse(ctx: &Context) -> Option<Vec<card::Ability>> {

    println!("parsing name {}", ctx.card_name);

    let mut abilities: Vec<card::Ability> = Vec::new();

    let mut is_mana_producer = false;
    let mut is_sac_for_cards = false;

    for (lhs, rhs) in ctx.text.split("\n")
        .filter(|line| line.contains(":"))
        .map(|line| {
            let v : Vec<&str> = line.splitn(2, ":").collect();
            (v[0].trim(), v[1].trim())
        }) {
        let cost = match parse_cost(lhs, ctx) {
            Err(error) => panic!("failed to parse oracle cost: {}; cost='{}', name='{}', text='{}'", error, lhs, ctx.card_name, ctx.text),
            Ok(value) => value
        };
        let effect = match parse_effect(rhs, ctx) {
            Err(error) => panic!("failed to parse oracle ability: {}; effect='{}', name='{}', text='{}'", error, rhs, ctx.card_name, ctx.text),
            Ok(value) => value
        };

        if cost.is_none() || effect.is_none() {
            continue;
        }

        let effect_ref = &effect.as_ref().unwrap();
        let cost_ref = &cost.as_ref().unwrap();

        is_mana_producer |= effect_ref.is_produce_mana();
        is_sac_for_cards |= effect_ref.is_draw() && cost_ref.is_sacrifice();

        let ability = card::Ability {
            trigger: card::Trigger::Activated,
            availability: 1.0,
            cost: cost.unwrap(),
            effect: effect.unwrap()
        };

        abilities.push(ability);
    }

    if is_mana_producer && is_sac_for_cards {
        abilities.iter_mut()
            .filter(|a| a.effect.is_draw())
            .for_each(|a| a.availability = 0.2);
    }

    return match abilities.len() {
        0 => None,
        _ => Some(abilities)
    }
}

fn parse_cost(cost_string: &str, ctx: &Context) -> Result<Option<card::Cost>, String> {
    lazy_static! {
        static ref RE_TAP: Regex = Regex::new(r"\{T\}").unwrap();
        static ref RE_SACRIFICE: Regex = Regex::new(r"^Sacrifice ([\w\s']+)").unwrap();
    }
    if RE_TAP.is_match(cost_string) {
        return Ok(Some(card::Cost::Tap));
    } else if let Some(cap) = RE_SACRIFICE.captures(cost_string) {
        if &cap[1] == ctx.card_name {
            return Ok(Some(card::Cost::Sacrifice));
        }
    }

    return Ok(None);
}

fn parse_effect(effect_string: &str, _ctx: &Context) -> Result<Option<card::Effect>, String> {
    lazy_static! {
        static ref ADD_MANA_X: Regex = Regex::new(r"^Add ((\{\w\})+)\.").unwrap();
        static ref ADD_MANA_X_OR_Y: Regex = Regex::new(r"^Add \{(\w)\} or \{(\w)\}.").unwrap();
        static ref ADD_MANA_COMMANDER: Regex = Regex::new("Add one mana of any color in your commander's color identity.").unwrap();
        static ref DRAW_A_CARD: Regex = Regex::new("^Draw a card.$").unwrap();
    }

    if let Some(cap) = ADD_MANA_X.captures(effect_string) {
        let pool = mana::ManaPool::new_from_string(&cap[1])?;
        return Ok(Some(card::Effect::ProduceMana(pool)));

    } else if let Some(cap) = ADD_MANA_X_OR_Y.captures(effect_string) {
        let mut mana = mana::Mana::new();
        mana.set_from_string(&cap[1])?;
        mana.set_from_string(&cap[2])?;
        return Ok(Some(card::Effect::ProduceMana(mana::ManaPool::new_from_single(&mana))));

    } else if ADD_MANA_COMMANDER.is_match(effect_string) {
        return Ok(Some(card::Effect::ProduceMana(mana::ManaPool::new_from_single(&mana::ALL))));

    } else if DRAW_A_CARD.is_match(effect_string) {
        return Ok(Some(card::Effect::Draw(vec![1])));

    }

    return Ok(None);
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_oracle_parse() {
        let thran_dynamo_text = "{T}: Add {C}{C}{C}.";
        match parse(&Context { text: thran_dynamo_text, card_name: "Thran Dynamo" }) {
            None => assert!(false),
            Some(abilities) => {
                assert_eq!(abilities.len(), 1);
                let ability = &abilities[0];
                assert_eq!(ability.trigger, card::Trigger::Activated);
                assert_eq!(ability.cost, card::Cost::Tap);
                assert_eq!(ability.effect, card::Effect::ProduceMana(mana::ManaPool::new_from_sequence(&vec![mana::COLORLESS, mana::COLORLESS, mana::COLORLESS])));
            }
        }

        let boros_cluestone_text = "{T}: Add {R} or {W}.";
        match parse(&Context { text: boros_cluestone_text, card_name: "Boros Cluestone" }) {
            None => assert!(false),
            Some(abilities) => {
                assert_eq!(abilities.len(), 1);
                let ability = &abilities[0];
                assert_eq!(ability.trigger, card::Trigger::Activated);
                assert_eq!(ability.cost, card::Cost::Tap);
                assert_eq!(ability.effect, card::Effect::ProduceMana(mana::ManaPool::new_from_single(&mana::Mana::make_dual(mana::Color::White, mana::Color::Red))));
            }
        }

        let commanders_sphere_text = "{T}: Add one mana of any color in your commander's color identity.\nSacrifice Commander's Sphere: Draw a card.";
        let results = parse(&Context { text: commanders_sphere_text, card_name: "Commander's Sphere" });
        println!("commanders sphere: \n {:?}", results);
    }
}
