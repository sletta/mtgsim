use crate::zone::*;
use crate::card::*;
use crate::mana::*;
use itertools::Itertools;
// use rand::Rng;
// use rand::distributions::{Distribution, Uniform};

// use std::cmp::Ordering;

#[derive(Clone)]
pub struct Game<'db> {
    pub library: Zone<'db>,
    pub hand: Zone<'db>,
    pub command: Zone<'db>,
    pub battlefield: Zone<'db>,
    pub graveyard: Zone<'db>,

    pub verbose: bool,
    pub game_stats : GameStats,
}

pub enum MulliganType {
    None,
    ThreeLands
}

pub struct Settings {
    pub draw_card_on_turn_one: bool,
    pub turn_count: u32,
    pub mulligan : MulliganType,
}

struct Turn<'db, 'game> {
    game: &'game mut Game<'db>,
    turn_number: u32,
    lands_played : u32,
    land_limit: u32,
    mana_pool : ManaPool,
    mana_spent : ManaPool,
    turn_stats : TurnStats,
    cards_in_mana_pool: std::collections::HashSet<u32>
}

#[derive(Debug, Clone)]
pub struct TurnStats {
    pub turn_number: u32,
    pub lands_played: u32,
    pub lands_cheated: u32,
    pub cards_drawn: u32,
    pub cards_played: u32,
    pub cards_in_hand: u32,
    pub mana_available: u32,
    pub mana_spent: u32,
}

#[derive(Debug, Clone)]
pub struct GameStats {
    pub mulligan_count: u32,
    pub turn_commander_played : u32,
    pub turns_stats : Vec<TurnStats>,
}

impl<'db> Game<'db> {
    pub fn new() -> Self {
        return Game {
            library: Zone::new("Library"),
            hand: Zone::new("Hand"),
            command: Zone::new("Command"),
            battlefield: Zone::new("Battlefield"),
            graveyard: Zone::new("Graveyard"),
            verbose: false,
            game_stats : GameStats {
                mulligan_count: 0,
                turn_commander_played: 0,
                turns_stats: Vec::new()
            },
        };
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        self.library.dump();
        self.hand.dump();
        self.command.dump();
    }

    pub fn draw_cards(&mut self, count : u32) {
        for _i in 0..count {
            let card = self.library.draw().expect("library is out of cards!!!");
            if self.verbose {
                println!(" - draw card: {}", card);
            }
            self.hand.add(card);
        }
    }

    fn draw_and_mulligan(&mut self, settings: &Settings) {
        match settings.mulligan {
            MulliganType::ThreeLands => {
                let pips = self.library.count_pips_in_mana_costs();
                let mut npips = pips.clone();
                npips.normalize();
                let color_average = (npips.black + npips.blue + npips.green + npips.red + npips.white) / 5.0;
                let mut primary_colors : Vec<Color> = Vec::new();
                if npips.black > color_average { primary_colors.push(Color::Black); }
                if npips.blue > color_average { primary_colors.push(Color::Blue); }
                if npips.green > color_average { primary_colors.push(Color::Green); }
                if npips.red > color_average { primary_colors.push(Color::Red); }
                if npips.white > color_average { primary_colors.push(Color::White); }
                if self.verbose {
                    println!("Primary deck colors: {:?}", primary_colors);
                    println!("Distribution of pips:");
                    println!(" - black ..: {:.2} / {:.2}", pips.black, npips.black);
                    println!(" - green ..: {:.2} / {:.2}", pips.green, npips.green);
                    println!(" - red ....: {:.2} / {:.2}", pips.red, npips.red);
                    println!(" - blue ...: {:.2} / {:.2}", pips.blue, npips.blue);
                    println!(" - white ..: {:.2} / {:.2}", pips.white, npips.white);
                }

                let original_library = self.library.clone();
                self.library.shuffle();
                self.draw_cards(7);
                while self.hand.query(Types::Land).len() < 3 {
                    if self.verbose {
                        println!("Not enough lands in hand, doing a mulligan...");
                        // self.hand.dump();
                    }
                    self.library = original_library.clone();
                    self.library.shuffle();
                    self.hand.clear();
                    self.draw_cards(7);
                    self.game_stats.mulligan_count += 1;
                }
            },
            MulliganType::None => {
                self.library.shuffle();
                self.draw_cards(7);
            }
        }
    }

    pub fn play(&mut self, settings: &Settings) {
        assert!(self.library.size() > 0);
        assert_eq!(self.hand.size(), 0);
        assert_eq!(self.battlefield.size(), 0);
        assert_eq!(self.graveyard.size(), 0);

        self.command.sort_by_cmc();

        let id = self.command.assign_ids(1);
        self.library.assign_ids(id);

        self.draw_and_mulligan(settings);

        for i in 0..settings.turn_count {

            let turn_stats;
            {
                let mut turn = Turn::new(self, i + 1);
                turn.play(&settings);
                turn_stats = turn.turn_stats;
            }

            self.game_stats.turns_stats.push(turn_stats);
        }
    }
}

impl<'db, 'game> Turn<'db, 'game> {

    pub fn new(game : &'game mut Game<'db>, turn : u32) -> Self {
        return Turn {
            game: game,
            turn_number: turn,
            lands_played: 0,
            land_limit: 1,
            mana_pool: ManaPool::new(),
            mana_spent: ManaPool::new(),
            turn_stats: TurnStats {
                turn_number: turn,
                lands_played: 0,
                lands_cheated: 0,
                cards_drawn: 0,
                cards_played: 0,
                cards_in_hand: 0,
                mana_available: 0,
                mana_spent: 0
            },
            cards_in_mana_pool: std::collections::HashSet::new()
        }
    }

    pub fn play(&mut self, settings: &Settings) {
        if self.game.verbose {
            println!("\n********** Turn #{} **********", self.turn_number);
        }

        self.game.battlefield.untap_all();

        // gather mana pool from lands, rocks and dorks
        for (card, ability) in self.find_abilities_on_battlefield(|ability| {
            ability.trigger.is_activated()
            && ability.cost.is_tap()
            && ability.effect.is_produce_mana()
            && (ability.availability == 1.0 || rand::random::<f32>() < ability.availability)
        }) {
            match &ability.effect {
                Effect::ProduceMana(mana) => self.add_to_mana_pool(&card, mana),
                _ => panic!("invalid effect when build mana pool...")
            }
        }
        if self.game.verbose {
            println!(" - available mana: {} ({})", self.mana_pool, self.mana_pool.cmc());
            println!();
        }

        // other upkeep stuff
        for (card, ability) in self.find_abilities_on_battlefield(|ability| {
            ability.trigger.is_upkeep()
            && ability.cost.is_none()
            && (ability.availability == 1.0 || rand::random::<f32>() < ability.availability)
        }) {
            match &ability.effect {
                Effect::ProduceMana(mana) => self.add_to_mana_pool(&card, mana),
                Effect::FetchLand{ to_hand: hand, to_battlefield: bf } => self.fetch_lands(hand, bf),
                Effect::Draw(ratios) => self.draw_cards(&card, ratios),
                _ => ()
            }
        }

        // draw card for turn..
        if self.turn_number > 1 || settings.draw_card_on_turn_one {
            if self.game.verbose {
                println!(" - drawing card for turn:");
            }
            self.game.draw_cards(1);
            self.turn_stats.cards_drawn += 1;
        }

        if self.game.verbose {
            println!("");
        }
        while self.try_to_play_land()
            || self.try_to_play_commander()
            || self.try_to_activate_ramp_ability()
            || self.try_to_play_ramp_spell()
            || self.try_to_activate_draw_spell()
            || self.try_to_play_draw_spell()
            || self.try_to_empty_hand()
            {
            if self.game.verbose {
                println!("");
            }
            continue;
        }

        if self.game.verbose {
            println!("Mana available: {} ({})", self.mana_pool, self.mana_pool.cmc());
            println!("Mana spent: {} ({})", self.mana_spent, self.mana_spent.cmc());
            self.game.hand.dump();
            self.game.battlefield.sort();
            self.game.battlefield.dump();
            println!();
        }

        self.turn_stats.mana_available = self.mana_pool.cmc();
        self.turn_stats.mana_spent = self.mana_spent.cmc();
        self.turn_stats.cards_in_hand = self.game.hand.size();
    }

    fn find_abilities_on_battlefield<F>(&self, selector: F) -> Vec<(Card<'db>, &'db Ability)> where F: Fn(&Ability) -> bool {

        let mut result : Vec<(Card, &Ability)> = Vec::new();
        for card in &self.game.battlefield.cards {
            for ability in card.data.abilities.iter().flatten() {
                if !selector(ability) {
                    continue;
                }

                // If it requires tapping, skip if we're already tapped.
                if ability.cost.is_tap() && card.tapped {
                    continue;
                }

                // Finally, check the mana cost. A little gotcha here is that
                // some cards are both tapped as activated abilities and
                // tapped for mana, so for these cards, we've already added
                // their mana to the mana pool, so we need to take it back
                // out account when checking if we can afford to pay it.
                if let Some(ability_cost) = ability.cost.is_mana() {
                    let mut mana_pool = self.mana_pool.clone();
                    if  self.cards_in_mana_pool.contains(&card.id) {
                        if let Some(produced) = card.produced_mana() {
                            mana_pool.remove_exact_pool(&produced);
                        }
                    }
                    if mana_pool.can_also_pay_for(&self.mana_spent, &ability_cost) == None {
                        continue;
                    }
                }

                // All good, lets include this ability
                result.push((card.clone(), ability));
            }
        }
        return result;
    }

    pub fn find_spells_in_hand<F>(&self, selector: F) -> Vec<Card<'db>> where F : Fn(&Ability) -> bool {
        let mut result: Vec<Card> = Vec::new();
        for card in &self.game.hand.cards {
            for ability in card.data.abilities.iter().flatten() {
                if card.is_type(Types::Land) {
                    continue;
                }

                if !selector(ability) {
                    continue;
                }
                if let Some(casting_cost) = &card.data.mana_cost {
                    if self.mana_pool.can_also_pay_for(&self.mana_spent, &casting_cost) == None {
                        continue;
                    }
                }
                result.push(card.clone());
            }
        }
        return result;
    }

    pub fn try_to_play_commander(&mut self) -> bool {
        if self.game.command.size() == 0 {
            return false;
        }
        let commander = &self.game.command.cards[0];
        let commander_cost = commander.data.mana_cost.as_ref().expect("commander has no mana cost!!!");
        match self.mana_pool.can_also_pay_for(&self.mana_spent, &commander_cost) {
            Some(spent) => {
                if self.game.verbose {
                    println!(" - playing commander, {}", commander);
                    println!("   -> to battlefield");
                }
                self.game.game_stats.turn_commander_played = self.turn_number;
                self.turn_stats.cards_played += 1;
                self.mana_spent = spent;
                let card = self.game.command.take(commander.id).expect("commander wasn't there!!!");
                self.game.battlefield.add(card);
                return true;
            },
            None => return false
        }
    }

    pub fn try_to_play_land(&mut self) -> bool {

        if self.lands_played >= self.land_limit {
            return false;
        }

        let mut lands_in_hand = self.game.hand.query(Types::Land);
        if lands_in_hand.len() == 0 {
            return false;
        }

        if self.game.verbose {
            println!(" - trying to play lands, {} in hand", lands_in_hand.len());
        }

        sort_cards_on_colors_produced(&mut lands_in_hand);

        let maybe_wanted_color = Self::evaluate_desired_mana_colors(&self.game.hand, &self.mana_pool);
        if let Some(wanted_color) = maybe_wanted_color {
            if self.game.verbose && wanted_color.len() > 0 {
                println!(" - land preference: {:?}", wanted_color[0]);
            }
            for color in wanted_color {
                for land in &lands_in_hand {
                    match &land.data.produced_mana {
                        Some(mana) => if mana.contains(color) {
                            let card = self.game.hand.take(land.id).unwrap();
                            self.play_card(card);
                            return true;
                        },
                        None => ()
                    }
                }
            }
        }

        if self.game.verbose {
            println!(" - no match for preference...");
        }

        let card = self.game.hand.take(lands_in_hand[0].id).unwrap();
        self.play_card(card);
        return true;
    }

    fn try_to_activate_ramp_ability(&mut self) -> bool {
        let mut abilities = self.find_abilities_on_battlefield(|ability| {
            ability.trigger.is_activated()
            && ability.effect.is_fetch_land()
            && ability.availability >= rand::random::<f32>()
        });
        if abilities.is_empty() {
            return false;
        }
        abilities.sort_by(|(_, ability_a), (_, ability_b)| {
            let cost_a = ability_a.cost.is_mana().map_or(0, |cost| cost.cmc());
            let cost_b = ability_b.cost.is_mana().map_or(0, |cost| cost.cmc());
            cost_a.cmp(&cost_b)
        });
        if self.game.verbose {
            for (card, ability) in &abilities {
                println!(" - activated ramp candidate: {} :: {}", card, ability);
            }
        }

        let (card, ability) = &abilities[0];
        if self.game.verbose {
            println!(" - activating {} :: {}", card, ability);
        }

        match &ability.effect {
            Effect::FetchLand{to_hand: types_to_hand, to_battlefield: types_to_battlefield} => {
                assert!(!ability.cost.is_tap() || !card.tapped);
                self.fetch_lands(types_to_hand, types_to_battlefield);
            },
            _ => panic!("unhandled ability type!!!")
        }

        let card = self.game.battlefield.take(card.id).unwrap();
        self.pay_activation_cost(card, &ability.cost);
        return true;
    }

    fn try_to_play_ramp_spell(&mut self) -> bool {
        let mut candidates = self.find_spells_in_hand(|ability| {
            match &ability.effect {
                Effect::FetchLand{to_hand: _, to_battlefield: _} => true,
                Effect::ProduceMana(_) => true,
                Effect::LandLimit(_) => self.game.hand.cards.iter().any(|card| card.is_type(Types::Land)),
                _ => false
            }
        });
        if candidates.is_empty() {
            return false;
        }
        candidates.sort_by(|a, b| a.data.cmc.cmp(&b.data.cmc));
        if self.game.verbose {
            for card in &candidates {
                println!(" - ramp spell candidate: {}", card);
            }
        }

        let card = self.game.hand.take(candidates[0].id).unwrap();
        self.play_card(card);
        return true;
    }

    fn try_to_activate_draw_spell(&mut self) -> bool {
        let mut abilities = self.find_abilities_on_battlefield(|ability|
            ability.trigger.is_activated()
            && ability.availability >= rand::random::<f32>()
            && ability.effect.is_draw()
            && (!ability.cost.is_sacrifice() || self.lands_played == 0)
        );
        if abilities.is_empty() {
            return false;
        }
        abilities.sort_by(|(_, ability_a), (_, ability_b)| {
            let cost_a = ability_a.cost.is_mana().map_or(0, |cost| cost.cmc());
            let cost_b = ability_b.cost.is_mana().map_or(0, |cost| cost.cmc());
            cost_a.cmp(&cost_b)
        });
        if self.game.verbose {
            for (card, ability) in &abilities {
                println!(" - activated draw candidate: {} :: {}", card, ability);
            }
        }

        let (card, ability) = &abilities[0];
        if self.game.verbose {
            println!(" - activating {} :: {}", card, ability);
        }

        match &ability.effect {
            Effect::Draw(ratios) => self.draw_cards(&card, ratios),
            _ => panic!("unhandled ability type!!!")
        }

        let card = self.game.battlefield.take(card.id).unwrap();
        self.pay_activation_cost(card, &ability.cost);
        return true;
    }

    fn try_to_play_draw_spell(&mut self) -> bool {
        let mut candidates = self.find_spells_in_hand(|ability|
            ability.effect.is_draw()
            && ability.availability >= rand::random::<f32>()
        );
        if candidates.is_empty() {
            return false;
        }
        candidates.sort_by(|a, b| a.data.cmc.cmp(&b.data.cmc));
        if self.game.verbose {
            for card in &candidates {
                println!(" - draw spell candidate: {}", card);
            }
        }
        let card = self.game.hand.take(candidates[0].id).unwrap();
        self.play_card(card);
        return true;
    }

    fn try_to_empty_hand(&mut self) -> bool {
        let mut candidates : Vec<Card> = self.game.hand.cards.iter().filter(|card| {
            if card.is_type(Types::Land)
                || card.data.abilities.is_some() {
                return false;
            }
            if let Some(cost) = &card.data.mana_cost {
                if self.mana_pool.can_also_pay_for(&self.mana_spent, &cost).is_none() {
                    return false;
                }
            }
            return true;
        }).map(|c| c.clone()).collect();
        if candidates.is_empty() {
            return false;
        }
        // candidates.sort_by(|a, b| a.data.cmc.cmp(&b.data.cmc));
        candidates.sort_by(|a, b| b.data.cmc.cmp(&a.data.cmc));
        if self.game.verbose {
            for card in &candidates {
                println!(" - other spell candidates: {}", card);
            }
        }

        // try to find "the perfect match.."
        if candidates.len() <= 5 && false {
            for perm in candidates.iter().permutations(candidates.len()) {
                let mut cost = self.mana_spent.clone();

                let mut id : Option<u32> = None;

                for card in &perm {
                    if let Some(casting_cost) = &card.data.mana_cost {
                        cost.add_pool(&casting_cost);
                        if !self.mana_pool.can_pay_for(&cost) {
                            break;
                        }
                        if cost.cmc() == self.mana_pool.cmc() {
                            id = Some(card.id);
                            break;
                        }
                    }
                }

                if let Some(last_id_in_permutation) = id {
                    for card in perm {
                        self.turn_stats.cards_played += 1;
                        let card_in_hand = self.game.hand.take(card.id).unwrap();
                        self.play_card(card_in_hand.clone());
                        if card.id == last_id_in_permutation {
                            return true;
                        }
                    }
                }
            }
        }

        let card = self.game.hand.take(candidates[0].id).unwrap();
        self.play_card(card);
        return true;
    }

    fn play_card(&mut self, mut card: Card<'db>) {
        if card.data.enters_tapped {
            card.tapped = true;
        }
        if self.game.verbose {
            println!(" - playing: {}", card);
        }

        if card.is_type(Types::Land) {
            assert!(self.lands_played < self.land_limit);
            self.lands_played += 1;
            self.turn_stats.lands_played += 1;
        } else {
            self.turn_stats.cards_played += 1;
        }

        let permanent = !(card.is_type(Types::Instant) || card.is_type(Types::Sorcery));

        // Resolving card ability...
        for ability in card.data.abilities.iter().flatten() {
            match &ability.effect {
                Effect::ProduceMana(pool) => {
                    // Lands, mana rocks, mana dorks, etc..
                    if permanent
                        && ability.trigger.is_activated()
                        && (!ability.cost.is_tap() || !card.tapped)
                        && !ability.cost.is_mana().is_some() {
                        self.add_to_mana_pool(&card, pool);
                    } else if ability.trigger.is_cast() {
                        self.add_to_mana_pool(&card, pool);
                    }
                },
                Effect::FetchLand{to_hand: types_to_hand, to_battlefield: types_to_battlefield} => {
                    if ability.trigger.is_cast() && ability.cost.is_none() {
                        self.fetch_lands(types_to_hand, types_to_battlefield);
                    }
                },
                Effect::LandLimit(increase) => self.land_limit += increase,
                Effect::Draw(ratios) => {
                    if ability.trigger.is_cast() && ability.cost.is_none() {
                        self.draw_cards(&card, ratios);
                    }
                }
            }
        }

        // pay mana cost
        if let Some(mana_cost) = &card.data.mana_cost {
            match self.mana_pool.can_also_pay_for(&self.mana_spent, &mana_cost) {
                Some(new_mana_spent) => self.mana_spent = new_mana_spent,
                None => panic!("cannot pay for {}!!!", card)
            }
        }

        if permanent {
            if self.game.verbose {
                println!(" - {} -> battlefield!", card);
            }
            self.game.battlefield.add(card);
        } else {
            if self.game.verbose {
                println!(" - {} -> graveyard!", card);
            }
            self.game.graveyard.add(card);
        }
    }

    // Pays the activation cost for the given card and cost. The card has been
    // removed from the battlefield here and needs to be put back unless it
    // is sacrificed, in which case it goes to graveyard.
    fn pay_activation_cost(&mut self, mut card: Card<'db>, cost: &Cost) {
        if cost.is_tap() {
            assert!(!card.tapped);
            card.tapped = true;
        }
        if cost.is_sacrifice() {
            if let Some(mana_produced) = card.produced_mana() {
                self.mana_pool.remove_exact_pool(&mana_produced);
            }
            if self.game.verbose {
                println!(" - {} -> graveyard!", card);
            }
            self.game.graveyard.add(card);

        } else {
            // put the card back now we've modified it..
            self.game.battlefield.add(card);
        }
        if let Some(mana_cost) = cost.is_mana() {
            self.mana_spent.add_pool(&mana_cost);
            assert!(self.mana_pool.can_pay_for(&self.mana_spent));
        }
    }

    fn add_to_mana_pool(&mut self, card: &Card<'db>, mana_produced: &ManaPool) {
        self.mana_pool.add_pool(mana_produced);
        self.cards_in_mana_pool.insert(card.id);
        if self.game.verbose {
            println!(" - add to mana pool: {}, {}", mana_produced, card);
        }
    }

    fn draw_cards(&mut self, card: &Card<'db>, ratios: &Vec<u32>) {
        if self.game.verbose {
            println!(" - drawing cards from {}", card);
        }
        let index = rand::random::<usize>() % ratios.len();
        self.game.draw_cards(ratios[index]);
        self.turn_stats.cards_drawn += ratios[index];
    }

    fn fetch_lands(&mut self, types_to_hand: &Vec<String>, types_to_battlefield: &Vec<String>) {
        for type_to_hand in types_to_hand {
            let maybe_card = self.game.library.take_land(type_to_hand);
            if let Some(card) = maybe_card {
                if self.game.verbose {
                    println!(" - fetch to hand: {}", card);
                }
                self.game.hand.add(card);
            } else if self.game.verbose {
                println!(" - no cards of type='{}' in library, fetch to hand failed...", type_to_hand);
            }
        }
        for type_to_battlefield in types_to_battlefield {
            let maybe_card = self.game.library.take_land(type_to_battlefield);
            if let Some(mut card) = maybe_card {
                card.tapped = true;
                self.turn_stats.lands_cheated += 1;
                if self.game.verbose {
                    println!(" - fetch to battlefield {}", card);
                }
                self.game.battlefield.add(card);
            } else if self.game.verbose {
                println!(" - no cards of type='{}' in library, fetch to battlefield failed...", type_to_battlefield);
            }
        }
        self.game.library.shuffle();
    }

    fn evaluate_desired_mana_colors(zone: &Zone, mana_pool: &ManaPool) -> Option<Vec<Color>> {
        let mut pips_in_zone = zone.count_pips_in_mana_costs();
        let has_pips_in_zone = pips_in_zone.normalize();
        let mut pips_in_mana_pool = PipCounts::new();
        pips_in_mana_pool.count_in_pool(mana_pool);
        let has_pips_in_mana_pool = pips_in_mana_pool.normalize();

        if has_pips_in_zone {
            return Some(match has_pips_in_mana_pool {
                true => pips_in_zone.prioritized_delta(&pips_in_mana_pool),
                false => pips_in_zone.prioritized_delta(&PipCounts::new())
            });
        }

        return None;
    }
}

fn sort_cards_on_colors_produced(cards : &mut Vec<Card>) {
    cards.sort_by(|a, b| {
        let colors_in_a : u32 = match &a.data.produced_mana {
            Some(mana) => mana.color_count(),
            None => 0
        };
        let colors_in_b : u32 = match &b.data.produced_mana {
            Some(mana) => mana.color_count(),
            None => 0
        };
        return colors_in_b.cmp(&colors_in_a);
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_turn_gather_mana_pool() {
        let sol_ring_data = CardData::make_sol_ring_data();
        let swamp_data = CardData::make_swamp_data();
        let command_tower_data = CardData::make_command_tower_data();
        let commanders_sphere_data = CardData::make_commanders_sphere_data();
        let elk_data = CardData::make_elk_data();

        let mut game : Game = Game::new();
        game.battlefield.add(Card::new_with_id(1, &command_tower_data));
        game.battlefield.add(Card::new_with_id(2, &sol_ring_data));
        game.battlefield.add(Card::new_with_id(3, &swamp_data));
        game.battlefield.add(Card::new_with_id(4, &commanders_sphere_data));
        game.battlefield.add(Card::new_with_id(5, &elk_data));

        let mut turn = Turn::new(&mut game, 42);
        turn.gather_mana_pool();
        assert_eq!(turn.mana_pool.colorless, 2);
        assert_eq!(turn.mana_pool.all, 2);
        assert_eq!(turn.mana_pool.black, 1);
    }

    #[test]
    fn test_game_sort_cards_on_colors_produced() {
        let swamp_data = CardData::make_swamp_data();
        let command_tower_data = CardData::make_command_tower_data();
        let jungle_hollow_data = CardData::make_jungle_hollow_data();
        let elk_data = CardData::make_elk_data();

        let mut cards = vec![
            Card::new_with_id(1, &swamp_data),
            Card::new_with_id(2, &elk_data),
            Card::new_with_id(3, &command_tower_data),
            Card::new_with_id(4, &jungle_hollow_data),
        ];

        sort_cards_on_colors_produced(&mut cards);
        assert_eq!(cards[0].id, 3); // command tower
        assert_eq!(cards[1].id, 4); // jungle hollow
        assert_eq!(cards[2].id, 1); // swamp
        assert_eq!(cards[3].id, 2); // elk
    }
}

// Rules for playing land:

