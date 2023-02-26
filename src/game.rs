use crate::zone::Zone;
use crate::card::{Card, Types};
use crate::mana;
use rand::distributions::{Distribution, Uniform};

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
    land_limit: u32
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
        self.battlefield.add(card);
    }

    pub fn play(&mut self, settings: &Settings) {

        assert!(self.library.size() > 0);
        assert_eq!(self.hand.size(), 0);
        assert_eq!(self.battlefield.size(), 0);
        assert_eq!(self.graveyard.size(), 0);

        let mut id = self.command.assign_ids(1);
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
        }
    }

    pub fn play(&mut self, settings: &Settings) {
        println!("\nPlaying turn: {}\n", self.turn_number);

        // untap all
        self.game.battlefield.untap_all();

        // draw card for turn..
        if self.turn_number > 1 || settings.draw_card_on_turn_one {
            self.game.draw_cards(1);
        }

        while self.try_to_play_land() {
            continue;
        }

        // self.game.hand.sort();
        self.game.hand.dump();
        self.game.battlefield.sort();
        self.game.battlefield.dump();
    }

    pub fn try_to_play_land(&mut self) -> bool {
        if self.lands_played >= self.land_limit {
            return false;
        }

        let lands_in_hand = self.game.hand.query(Types::Land);
        if lands_in_hand.len() == 0 {
            return false;
        }

        let colors_in_play = self.game.battlefield.find_produced_colors();
        let mut colors_in_hand = self.game.hand.find_produced_colors();
        println!(" -> {} in place, {} in hand", colors_in_play, colors_in_hand);

        colors_in_hand.subtract(&colors_in_play);

        let mut id : Option<u32> = None;
        if colors_in_hand != mana::COLORLESS {
            println!(" -> colors {} is not in play, try to play that...", colors_in_hand);
            for i in lands_in_hand.iter() {
                match &i.data.produced_mana {
                    Some(mana) => {
                        if mana.can_pay_for(&colors_in_hand) {
                            id = Some(i.id);
                        }
                    },
                    None => { }
                }
            }
        } else {
            id = Some(lands_in_hand[0].id);
        }

        if id.is_some() {
            self.game.play_card(id.unwrap());
            self.lands_played += 1;
            return true;
        }

        return false;
    }

    // pub fn try_to_play_land(&mut self, game: &'a mut Game<'a>) -> bool {
    //     // let lands : Vec<Card> = self.game.hand.cards.iter().filter_map(|c| match c.is_land() {
    //     //     true => Some(c),
    //     //     false => None
    //     // }).collect();

    //     // println!(" -- found lands to play: {:?}". lands);

    //     // if lands.is_empty() {
    //         return false;
    //     // }
    //     // return true;
    // }

}
