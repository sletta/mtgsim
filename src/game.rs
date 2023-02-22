use crate::zone::Zone;
use crate::card;
// use crate::mana;

#[derive(Clone)]
pub struct Game {
    pub library: Zone,
    pub hand: Zone,
    pub command: Zone,
    pub battlefield: Zone,
    pub graveyard: Zone,
}

impl Game {
    pub fn new() -> Self {
        return Game {
            library: Zone::new("Library"),
            hand: Zone::new("Hand"),
            command: Zone::new("Command"),
            battlefield: Zone::new("Battlefield"),
            graveyard: Zone::new("Graveyard")
        };
    }

    pub fn setup(&mut self) {
        self.library.shuffle();
        self.draw_cards(7);
        self.hand.sort();
        self.dump();
    }

    pub fn draw_cards(&mut self, count : u32) {
        for _i in 0..count {
            self.hand.add(self.library.draw().expect("library is out of cards!!!"));
        }
    }

    pub fn dump(&self) {
        self.library.dump();
        self.hand.dump();
    }
}
