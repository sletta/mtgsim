use crate::card::{Card, Types};
use crate::mana;
use rand::distributions::{Distribution, Uniform};

#[derive(Debug, Clone)]
pub struct Zone<'db> {
    pub name: String,
    pub cards: Vec<Card<'db>>
}

#[derive(Debug)]
pub struct PipCounts {
    black: f32,
    blue: f32,
    green: f32,
    red: f32,
    white: f32,
}

impl<'db> Zone<'db> {

    pub fn new(name : &str) -> Self {
        return Self { name: name.to_string(), cards: Vec::new() };
    }

    pub fn size(&self) -> u32 {
        return self.cards.len() as u32;
    }

    pub fn assign_ids(&mut self, first_id : u32) -> u32 {
        let mut id = first_id;
        for card in self.cards.iter_mut() {
            card.id = id;
            id += 1;
        }
        return id;
    }

    pub fn add(&mut self, card : Card<'db>) {
        self.cards.push(card);
    }

    pub fn shuffle(&mut self) {
        let mut random_generator = rand::thread_rng();
        let random = Uniform::from(0..self.cards.len());
        for i in 0..self.cards.len() {
            let pos = random.sample(&mut random_generator);
            self.cards.swap(i, pos);
        }
    }

    pub fn untap_all(&mut self) {
        self.cards.iter_mut().for_each(|c| c.tapped = false);
    }

    pub fn draw(&mut self) -> Option<Card<'db>> {
        return self.cards.pop();
    }

    pub fn take(&mut self, id : u32) -> Option<Card<'db>> {
        let mut i = 0;
        while i < self.cards.len() {
            if self.cards[i].id == id {
                return Some(self.cards.remove(i));
            }
            i += 1;
        }
        return None;
    }

    pub fn sort(&mut self) {
        self.cards.sort_by(|a, b| a.data.cmc.cmp(&b.data.cmc));
    }

    pub fn query(&self, card_type: Types) -> Vec<Card<'db>> {
        return self.cards.iter().filter_map(|c| match c.is_type(card_type) {
            true => Some(c.clone()),
            false => None
        }).collect();
    }

    pub fn find_produced_colors(&self) -> mana::Mana {
        let mut colors_in_zone = mana::Mana::new();
        self.cards.iter().for_each(|c| {
            if c.is_type(Types::Land) {
                match &c.data.produced_mana {
                    Some(colors) => colors_in_zone.unite(&colors),
                    None => { }
                }
            }
        });
        return colors_in_zone;
    }

    pub fn count_pips_in_mana_costs(&self) -> PipCounts {
        let mut pip_counts = PipCounts::new();
        for c in self.cards.iter() {
            match &c.data.mana_cost {
                Some(cost) => pip_counts.count_in_pool(cost),
                None => ()
            }
        }
        return pip_counts;
    }

    pub fn dump(&self) {
        println!("Zone: {}, {} cards", self.name, self.cards.len());
        for card in self.cards.iter() {
            println!("   {}", card);
        }
    }
}

impl PipCounts {
    pub fn new() -> Self {
        return PipCounts { black: 0.0, blue: 0.0, green: 0.0, red: 0.0, white: 0.0 };
    }

    pub fn count_in_mana(&mut self, m : &mana::Mana) {
        if m.contains(mana::Color::Black) {
            self.black += 1.0;
        }
        if m.contains(mana::Color::Blue) {
            self.blue += 1.0;
        }
        if m.contains(mana::Color::Green) {
            self.green += 1.0;
        }
        if m.contains(mana::Color::Red) {
            self.red += 1.0;
        }
        if m.contains(mana::Color::White) {
            self.white += 1.0;
        }
    }

    pub fn count_in_pool(&mut self, pool : &mana::Pool) {
        for m in pool.sequence.iter() {
            self.count_in_mana(&m);
        }
    }

    pub fn normalize(&mut self) {
        let sum = self.black + self.blue + self.green + self.red + self.white;
        self.black /= sum;
        self.blue /= sum;
        self.green /= sum;
        self.red /= sum;
        self.white /= sum;
    }

}

