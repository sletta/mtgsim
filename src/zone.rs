use crate::card::Card;
use rand::distributions::{Distribution, Uniform};

#[derive(Debug, Clone)]
pub struct Zone {
    name: String,
    cards: Vec<Card>
}

impl Zone {

    pub fn new(name : &str) -> Self {
        return Self { name: name.to_string(), cards: Vec::new() };
    }

    pub fn add(&mut self, card : Card) {
        self.cards.push(card);
    }

    pub fn shuffle(&mut self) {
        let mut random_generator = rand::thread_rng();
        let random = Uniform::from(0..self.cards.len());
        println!("shuffle cardpile: size={}", self.cards.len());
        for i in 0..self.cards.len() {
            let pos = random.sample(&mut random_generator);
            self.cards.swap(i, pos);
        }
    }

    pub fn draw(&mut self) -> Option<Card> {
        return self.cards.pop();
    }

    pub fn sort(&mut self) {
        self.cards.sort_by(|a, b| a.cmc.cmp(&b.cmc));
    }

    pub fn dump(&self) {
        println!("Zone: {}, {} cards", self.name, self.cards.len());
        for card in self.cards.iter() {
            println!("   {:?}", card);
        }
    }
}

