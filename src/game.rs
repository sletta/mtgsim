use crate::zone::Zone;
// use crate::card;
// use crate::mana;

#[derive(Clone)]
pub struct Game<'a> {
    pub library: Zone<'a>,
    pub hand: Zone<'a>,
    pub command: Zone<'a>,
    pub battlefield: Zone<'a>,
    pub graveyard: Zone<'a>,
}

impl<'a> Game<'a> {
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
            let card = &self.library.draw().expect("library is out of cards!!!");
            self.hand.add(card.clone());
        }
    }

    pub fn dump(&self) {
        self.library.dump();
        self.hand.dump();
        self.command.dump();
    }
}
