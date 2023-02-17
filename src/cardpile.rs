use crate::card::Card;
use rand::distributions::{Distribution, Uniform};

#[derive(Debug)]
pub struct CardPile {
    pub cards: Vec<Card>
}

impl CardPile {
    pub fn shuffle(&mut self) {
        let mut random_generator = rand::thread_rng();
        let random = Uniform::from(0..self.cards.len());
        println!("shuffle cardpile: size={}", self.cards.len());
        for i in 0..self.cards.len() {
            let pos = random.sample(&mut random_generator);
            self.cards.swap(i, pos);
        }
    }

    pub fn dump(&self) {
        println!("Dump of CardPile:");
        println!(" - # of cards: {}", self.cards.len());
        for card in self.cards.iter() {
            println!("   {:?}", card);
        }
        println!("----- End of Library -----");
    }
}

