use crate::zone::*;
use crate::card::*;
use crate::mana::*;
use rand::distributions::{Distribution, Uniform};

use std::cmp::Ordering;

#[derive(Clone)]
pub struct Game<'db> {
    pub library: Zone<'db>,
    pub hand: Zone<'db>,
    pub command: Zone<'db>,
    pub battlefield: Zone<'db>,
    pub graveyard: Zone<'db>,
}

pub struct Settings {
    pub draw_card_on_turn_one: bool,
    pub turn_count: u32,
}

struct Turn<'db, 'game> {
    game: &'game mut Game<'db>,
    turn_number: u32,
    lands_played : u32,
    land_limit: u32,
    mana_pool : Option<Pool>,
}

impl<'db> Game<'db> {
    pub fn new() -> Self {
        return Game {
            library: Zone::new("Library"),
            hand: Zone::new("Hand"),
            command: Zone::new("Command"),
            battlefield: Zone::new("Battlefield"),
            graveyard: Zone::new("Graveyard")
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
            self.hand.add(card);
        }
    }

    pub fn play_card(&mut self, id : u32) {
        let mut card = self.hand.take(id).expect("card missing from hand!!!");
        if card.data.enters_tapped {
            card.tapped = true;
        }
        println!(" -> playing: {}", card);
        self.battlefield.add(card);

    }

    pub fn gather_mana_pool(&self) -> Pool {
        let mut pool = Pool::new();
        for (card, abilities) in self.battlefield.cards.iter().filter_map(|c| Some((c, c.data.abilities.as_ref()?))) {
            if card.tapped {
                continue;
            }
            add_mana_production_to_pool(&abilities, &mut pool);
        }
        return pool;
    }

    pub fn play(&mut self, settings: &Settings) {

        assert!(self.library.size() > 0);
        assert_eq!(self.hand.size(), 0);
        assert_eq!(self.battlefield.size(), 0);
        assert_eq!(self.graveyard.size(), 0);

        let id = self.command.assign_ids(1);
        self.library.assign_ids(id);

        self.library.shuffle();
        self.draw_cards(7);

        for i in 0..settings.turn_count {
            let mut turn = Turn::new(self, i + 1);
            turn.play(&settings);
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
            mana_pool: None,
        }
    }

    pub fn play(&mut self, settings: &Settings) {
        println!("\nPlaying turn: {}\n", self.turn_number);

        // untap all
        self.game.battlefield.untap_all();

        // gather mana
       self.mana_pool = Some(self.game.gather_mana_pool());

        // draw card for turn..
        if self.turn_number > 1 || settings.draw_card_on_turn_one {
            self.game.draw_cards(1);
        }


        while self.try_to_play_land() {
            continue;
        }

        // self.game.hand.sort();
        println!("Mana Pool: {}", self.mana_pool.as_ref().unwrap());
        self.game.hand.dump();
        self.game.battlefield.sort();
        self.game.battlefield.dump();
    }

    pub fn try_to_play_land(&mut self) -> bool {
        if self.lands_played >= self.land_limit {
            return false;
        }

        let mut lands_in_hand = self.game.hand.query(Types::Land);
        if lands_in_hand.len() == 0 {
            // try to draw lands?
            println!(" -> no lands in hand, so not playing any...");
            return false;
        }

        sort_cards_on_colors_produced(&mut lands_in_hand);

        let mut pips_in_hand = self.game.hand.count_pips_in_mana_costs();
        let has_pips_in_hand = pips_in_hand.normalize();
        let mut pips_in_mana_pool = PipCounts::new();
        pips_in_mana_pool.count_in_pool(self.mana_pool.as_ref().unwrap());
        let has_pips_in_mana_pool = pips_in_mana_pool.normalize();

        println!(" -> in-hand={:?}, in-pool={:?}, ", pips_in_hand, pips_in_mana_pool);

        if has_pips_in_hand && has_pips_in_mana_pool {
            let wanted_color = pips_in_hand.prioritized_delta(&pips_in_mana_pool);
            println!(" -> wanted: {:?}", wanted_color);
            for color in wanted_color {
                for land in &lands_in_hand {
                    match &land.data.produced_mana {
                        Some(mana) => if mana.contains(color) {
                            self.play_land(land);
                            return true;
                        },
                        None => ()
                    }
                }
            }
        }

        println!(" -> giving up, just playing: {}", lands_in_hand[0]);
        self.play_land(&lands_in_hand[0]);
        return true;
    }

    fn play_land(&mut self, card : &Card<'db>) {
        self.lands_played += 1;
        assert!(self.lands_played <= self.land_limit);

        self.game.play_card(card.id);

        if let Some(pool) = self.mana_pool.as_mut() {
            match &card.data.abilities {
                Some(abilities) => add_mana_production_to_pool(&abilities, pool),
                None => ()
            }
        }
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

fn add_mana_production_to_pool(abilities : &Vec<Ability>, pool : &mut Pool)
{
    for ability in abilities.iter() {
        match &ability.trigger { Trigger::Activated => (), _ => continue }
        match &ability.cost { Cost::Tap => (), _ => continue }
        match &ability.effect {
            Effect::ProduceMana(produced_mana) => pool.add(produced_mana),
            _ => continue
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_game_gather_mana_pool() {
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

        let pool = game.gather_mana_pool();
        assert_eq!(pool.count(&COLORLESS), 2);
        assert_eq!(pool.count(&ALL), 2);
        assert_eq!(pool.count(&BLACK), 1);
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

