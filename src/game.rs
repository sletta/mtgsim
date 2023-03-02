use crate::zone::*;
use crate::card::*;
use crate::mana::*;
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
    mana_pool : Pool,
    mana_spent : Pool,
    turn_stats : TurnStats,
    cards_in_mana_pool: std::collections::HashSet<u32>
}

#[derive(Debug, Clone)]
pub struct TurnStats {
    pub turn_number: u32,
    pub lands_played: u32,
    pub cards_played: u32,
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
            mana_pool: Pool::new(),
            mana_spent: Pool::new(),
            turn_stats: TurnStats {
                turn_number: turn,
                lands_played: 0,
                cards_played: 0,
            },
            cards_in_mana_pool: std::collections::HashSet::new()
        }
    }

    pub fn play(&mut self, settings: &Settings) {
        self.game.battlefield.untap_all();
        self.gather_mana_pool();

        if self.game.verbose {
            println!("\n***** Turn #{} *****", self.turn_number);
            println!(" - available mana: {} ({})", self.mana_pool, self.mana_pool.converted_mana_cost());
        }

        if self.game.verbose {
        }

        // draw card for turn..
        if self.turn_number > 1 || settings.draw_card_on_turn_one {
            self.game.draw_cards(1);
        }

        while self.try_to_play_land()
            || self.try_to_play_commander()
            || self.try_to_ramp() {
            continue;
        }

        if self.game.verbose {
            println!(" - nothing more to do...");
            println!(" - available mana: {} ({})", self.mana_pool, self.mana_pool.converted_mana_cost());
            println!(" - spent mana: {} ({})", self.mana_spent, self.mana_spent.converted_mana_cost());
            self.game.hand.dump();
            self.game.battlefield.sort();
            self.game.battlefield.dump();
        }
    }

    pub fn gather_mana_pool(&mut self) {
        for (card, abilities) in self.game.battlefield.cards.iter().filter_map(|c| Some((c, c.data.abilities.as_ref()?))) {
            if card.tapped {
                continue;
            }
            if add_permanents_mana_production_to_pool(&abilities, &mut self.mana_pool) {
                self.cards_in_mana_pool.insert(card.id);
            }
        }
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
            // try to draw lands?
            return false;
        }

        if self.game.verbose {
            println!(" - trying to play lands, {} in hand", lands_in_hand.len());
        }

        sort_cards_on_colors_produced(&mut lands_in_hand);

        let mut pips_in_hand = self.game.hand.count_pips_in_mana_costs();
        let has_pips_in_hand = pips_in_hand.normalize();
        let mut pips_in_mana_pool = PipCounts::new();
        pips_in_mana_pool.count_in_pool(&self.mana_pool);
        let has_pips_in_mana_pool = pips_in_mana_pool.normalize();

        if has_pips_in_hand {
            let wanted_color = match has_pips_in_mana_pool {
                true => pips_in_hand.prioritized_delta(&pips_in_mana_pool),
                false => pips_in_hand.prioritized_delta(&PipCounts::new())
            };

            if self.game.verbose && wanted_color.len() > 0 {
                println!(" - land preference: {:?}", wanted_color[0]);
            }

            for color in wanted_color {
                for land in &lands_in_hand {
                    match &land.data.produced_mana {
                        Some(mana) => if mana.contains(color) {
                            self.play_card(land.id);
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
        self.play_card(lands_in_hand[0].id);
        return true;
    }

    pub fn can_satisfy_cost(&self, card: &Card<'db>, cost: &Cost) -> bool {

        if cost.is_tap() && card.tapped {
            return false;
        }

        if cost.is_sacrifice() && !self.game.battlefield.contains(card) {
            return false;
        }

        if let Some(pool) = cost.is_mana() {
            let mut total_mana_this_turn = self.mana_spent.expanded(&pool);
            if  self.cards_in_mana_pool.contains(&card.id) {
                if let Some(produced) = card.produced_mana() {
                    total_mana_this_turn.add(&produced);
                }
            }
            return self.mana_pool.can_pay_for(&total_mana_this_turn);
        }

        return true;
    }

    pub fn try_to_ramp(&mut self) -> bool {

        for card in self.game.battlefield.cards.iter() {
            for ability in card.data.abilities.iter().flatten() {
                match &ability.effect {
                    Effect::FetchLand{to_hand: types_to_hand, to_battlefield: types_to_battlefield} => {
                        if ability.trigger != Trigger::Activated
                            || !self.can_satisfy_cost(card, &ability.cost) {
                            continue;
                        }
                        if self.game.verbose {
                            println!(" - ramping (activated ability): {}", card);
                        }

                        if self.cards_in_mana_pool.contains(&card.id) {
                            self.cards_in_mana_pool.remove(&card.id);
                            if let Some(production) = card.produced_mana() {
                                self.mana_pool.remove(&production);
                                if self.game.verbose {
                                    println!(" - removing {} from mana pool", production);
                                }
                            }
                        }

                        if let Some(mana_cost) = ability.cost.is_mana() {
                            // registering mana spent..
                            self.mana_spent.add(mana_cost);
                        }

                        if ability.cost.is_sacrifice() {
                            println!(" - sac'ing {}", card);
                            let tmp_card = self.game.battlefield.take(card.id);
                            self.game.graveyard.add(tmp_card.expect("card missing!!!"));
                        }

                        fetch_lands(&mut self.game, types_to_hand, types_to_battlefield);
                        return true;
                    },
                    _ => ()
                }
            }
        }

        let mut candidates : Vec<Card> = Vec::new();
        for card in self.game.hand.cards.iter() {
            if card.is_type(Types::Land)
                || !card.is_ramp()
                || self.mana_spent.converted_mana_cost() + card.data.cmc > self.mana_pool.converted_mana_cost() {
                continue;
            }
            if self.game.verbose {
                println!(" - ramp candidate (cast): {}", card);
            }
            candidates.push(card.clone());
        }

        for card in candidates {
            if self.play_card_if_mana_allows(&card) {
                return true;
            }
        }

        return false;
    }

    pub fn play_card(&mut self, id : u32) {
        let mut card = self.game.hand.take(id).expect("card missing from hand!!!");
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
                Effect::ProduceMana(mana) => {
                    // Lands, mana rocks, mana dorks, etc..
                    if permanent
                        && !card.tapped
                        && ability.cost == Cost::Tap
                        && ability.trigger == Trigger::Activated {
                        if self.game.verbose {
                            println!(" - permanent's mana ability added to pool...");
                        }
                       self.mana_pool.add(mana);
                       self.cards_in_mana_pool.insert(card.id);
                    }
                },
                Effect::FetchLand{to_hand: types_to_hand, to_battlefield: types_to_battlefield} => {
                    if ability.trigger == Trigger::Cast && ability.cost == Cost::None {
                        fetch_lands(&mut self.game, types_to_hand, types_to_battlefield);
                    }
                },
                _ => (),

            }
        }

        if permanent {
            if self.game.verbose {
                println!(" - {} ---> battlefield", card);
            }
            self.game.battlefield.add(card);
        } else {
            if self.game.verbose {
                println!(" - {} ---> graveyard", card);
            }
            self.game.graveyard.add(card);
        }
    }

    fn play_card_if_mana_allows(&mut self, card : &Card<'db>) -> bool {

        let mut total_cost = self.mana_spent.clone();
        match &card.data.mana_cost {
            Some(casting_cost) => total_cost.add(&casting_cost),
            None => ()
        }

        if !self.mana_pool.can_pay_for(&total_cost) {
            if self.game.verbose {
                println!(" - cannot afford to play {}, total-cost={}, available={}", card, total_cost, self.mana_pool);
            }
            return false;
        }

        self.mana_spent = total_cost;
        self.play_card(card.id);

        return true;
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

fn add_permanents_mana_production_to_pool(abilities : &Vec<Ability>, pool : &mut Pool) -> bool {
    let mut added = false;
    for ability in abilities.iter() {
        match &ability.trigger { Trigger::Activated => (), _ => continue }
        match &ability.cost { Cost::Tap => (), _ => continue }
        match &ability.effect {
            Effect::ProduceMana(produced_mana) => {
                pool.add(produced_mana);
                added = true;
            },
            _ => continue
        }
    }
    return added;
}

fn fetch_lands(game: &mut Game, types_to_hand: &Vec<String>, types_to_battlefield: &Vec<String>) {
    for type_to_hand in types_to_hand {
        let card = game.library.take_land(type_to_hand).expect("land going to hand not found in library!!");
        if game.verbose {
            println!(" #--> fetch {} to hand", card);
        }
        game.hand.add(card);
    }
    for type_to_battlefield in types_to_battlefield {
        let mut card = game.library.take_land(type_to_battlefield).expect("land going to battlefield not found in library!!");
        card.tapped = true;
        if game.verbose {
            println!(" #--> fetch {} to battlefield", card);
        }
        game.battlefield.add(card);
    }
    game.library.shuffle();
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
        assert_eq!(turn.mana_pool.count(&COLORLESS), 2);
        assert_eq!(turn.mana_pool.count(&ALL), 2);
        assert_eq!(turn.mana_pool.count(&BLACK), 1);
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

