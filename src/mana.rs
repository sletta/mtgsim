#[allow(unused_imports)]
use itertools::Itertools;
#[allow(unused_imports)]
use std::time::{Duration, Instant};

use enumflags2::{bitflags, BitFlags};



#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black       = 1 << 0,
    Blue        = 1 << 1,
    Green       = 1 << 2,
    Red         = 1 << 3,
    White       = 1 << 4
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Mana {
    colors : BitFlags<Color>
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManaPool {
    pub black: u32,
    pub blue: u32,
    pub green: u32,
    pub red: u32,
    pub white: u32,
    pub colorless: u32,
    pub all: u32,
    pub multi: Option<Vec<Mana>> // make this optional
}

pub const COLORLESS : Mana  = Mana { colors: BitFlags::EMPTY };
pub const BLACK : Mana      = Mana { colors: enumflags2::make_bitflags!(Color::{ Black }) };
pub const BLUE : Mana       = Mana { colors: enumflags2::make_bitflags!(Color::{ Blue }) };
pub const GREEN : Mana      = Mana { colors: enumflags2::make_bitflags!(Color::{ Green }) };
pub const RED : Mana        = Mana { colors: enumflags2::make_bitflags!(Color::{ Red }) };
pub const WHITE : Mana      = Mana { colors: enumflags2::make_bitflags!(Color::{ White }) };

#[allow(dead_code)]
pub const ALL : Mana        = Mana { colors: enumflags2::make_bitflags!(Color::{ Black | Blue | Green | Red | White }) };

impl std::fmt::Display for Mana {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut colors = Vec::new();
        if self.colors.contains(Color::Black) {
            colors.push("B");
        }
        if self.colors.contains(Color::Blue) {
            colors.push("U");
        }
        if self.colors.contains(Color::Green) {
            colors.push("G");
        }
        if self.colors.contains(Color::Red) {
            colors.push("R");
        }
        if self.colors.contains(Color::White) {
            colors.push("W");
        }

        let mut text = String::new();
        match colors.len() {
            0 => text.push_str("{1}"),
            1 => {
                text.push_str("{");
                text.push_str(colors[0]);
                text.push_str("}");
            },
            _ => {
                text.push_str("{");
                for i in 0..colors.len() {
                    if i > 0 {
                        text.push_str("/");
                    }
                    text.push_str(colors[i]);
                }
                text.push_str("}");
            }
        }

        write!(f, "{}", text)
    }
}

impl Mana {
    pub fn new() -> Self {
        return COLORLESS.clone();
    }

    #[cfg(test)]
    pub fn make_mono(a : Color) -> Self {
        return Mana { colors: BitFlags::empty() | a };
    }

    #[cfg(test)]
    pub fn make_dual(a : Color, b : Color) -> Self {
        return Mana { colors: a | b };
    }

    #[cfg(test)]
    pub fn make_triple(a: Color, b: Color, c: Color) -> Self {
        return Mana { colors: a | b | c };
    }

    pub fn set_from_string(&mut self, value : &str) -> Result<(), String> {
        match value {
            "C" => {
                // do nothing..
            }
            "B" => self.colors |= Color::Black,
            "U" => self.colors |= Color::Blue,
            "G" => self.colors |= Color::Green,
            "R" => self.colors |= Color::Red,
            "W" => self.colors |= Color::White,
            _ => return Err(format!("bad mana string '{}'", value)),
        }
        Ok(())
    }

    pub fn is_colorless(&self) -> bool {
        return self.colors.is_empty();
    }

    pub fn is_monocolor(&self) -> bool {
        return self.colors.exactly_one() != None;
    }

    #[cfg(test)]
    pub fn can_pay_for(&self, other : &Mana) -> bool {
        return other.is_colorless() || self.colors.intersects(other.colors)
    }

    pub fn contains(&self, color : Color) -> bool {
        return self.colors.contains(color);
    }

    #[cfg(test)]
    pub fn can_pay_for_exactly(&self, other : &Mana) -> bool {
        return self.colors == other.colors && (self.is_colorless() || self.is_monocolor());
    }

    pub fn color_count(&self) -> u32 {
        return self.colors.len() as u32;
    }
}

impl std::fmt::Display for ManaPool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.cmc() == 0 {
            return write!(f, "n/a");
        }
        if self.colorless > 0 {
            write!(f, "{{{}}}", self.colorless).expect("formatting failed");
        }
        (0..self.black).for_each(|_| write!(f, "{{B}}").expect("formatting failed!"));
        (0..self.blue).for_each(|_| write!(f, "{{U}}").expect("formatting failed!"));
        (0..self.green).for_each(|_| write!(f, "{{G}}").expect("formatting failed!"));
        (0..self.red).for_each(|_| write!(f, "{{R}}").expect("formatting failed!"));
        (0..self.white).for_each(|_| write!(f, "{{W}}").expect("formatting failed!"));
        self.multi.iter().flatten().for_each(|m| write!(f, "{}", m).expect("formatting failed!"));
        (0..self.all).for_each(|_| write!(f, "{{B/U/G/R/W}}").expect("formatting failed!"));
        return Ok(());
    }
}


impl ManaPool {
    pub fn new() -> Self {
        return ManaPool {
            black: 0,
            blue: 0,
            green: 0,
            red: 0,
            white: 0,
            colorless: 0,
            all: 0,
            multi: None
        }
    }

    #[cfg(test)]
    pub fn new_from_sequence(s: &Vec<Mana>) -> Self {
        let mut pool = ManaPool::new();
        s.iter().for_each(|m| pool.add_mana(m));
        return pool;
    }

    pub fn new_from_string(cost : &str) -> Result<Self, String> {
        let re = regex::Regex::new(r"([0-9BCGRUW/]+)").expect("failed to crate manacost reggexp");
        let mut pool = ManaPool::new();
        for cap in re.find_iter(cost) {
            let value = cap.as_str();
            if value.contains("/") {
                let mut mana = Mana::new();
                for i in value.split('/') {
                    mana.set_from_string(i)?;
                }
                pool.add_mana(&mana);
                continue;
            }
            match value {
                "C" => pool.add_mana(&COLORLESS),
                "B" => pool.add_mana(&BLACK),
                "U" => pool.add_mana(&BLUE),
                "G" => pool.add_mana(&GREEN),
                "R" => pool.add_mana(&RED),
                "W" => pool.add_mana(&WHITE),
                _ => {
                    let count = value.parse::<u32>().unwrap_or_else(|e| panic!("failed to parse mana value! error={:?}, value='{:?}'", e, value));
                    for _i in 0..count {
                        pool.add_mana(&COLORLESS);
                    }
                }
            }
        }

        if pool.cmc() == 0 {
            return Err(format!("invalid mana cost... {:?}", cost));
        }

        return Ok(pool);
    }

    pub fn new_from_single(mana: &Mana) -> Self {
        let mut pool = ManaPool::new();
        pool.add_mana(mana);
        return pool;
    }


    pub fn add_mana(&mut self, m: &Mana) {
        if m.is_colorless() {
            self.colorless += 1;
        } else if m.is_monocolor() {
            if *m == BLACK {
                self.black += 1;
            } else if *m == BLUE {
                self.blue += 1;
            } else if *m == GREEN {
                self.green += 1;
            } else if *m == RED {
                self.red += 1;
            } else if *m == WHITE {
                self.white += 1;
            }
        } else if *m == ALL {
            self.all += 1;
        } else {
            self.multi.get_or_insert(Vec::new()).push(m.clone());
        }
    }

    pub fn add_color(&mut self, color: &Color) {
        match color {
            Color::Black => self.black += 1,
            Color::Blue => self.blue += 1,
            Color::Green => self.green += 1,
            Color::Red => self.red += 1,
            Color::White => self.white+= 1,
        }
    }

    pub fn add_pool(&mut self, pool: &ManaPool) {
        self.black += pool.black;
        self.blue += pool.blue;
        self.green += pool.green;
        self.red += pool.red;
        self.white += pool.white;
        self.colorless += pool.colorless;
        self.all += pool.all;
        pool.multi
            .iter()
            .flatten()
            .for_each(|m| self.multi.get_or_insert(Vec::new()).push(m.clone()));
    }

    pub fn remove_exact_pool(&mut self, other: &ManaPool) {
        assert!(self.black >= other.black);
        assert!(self.blue >= other.blue);
        assert!(self.green >= other.green);
        assert!(self.red >= other.red);
        assert!(self.white >= other.white);
        assert!(self.colorless >= other.colorless);
        assert!(self.all >= other.all);

        self.black -= other.black;
        self.blue -= other.blue;
        self.green -= other.green;
        self.red -= other.red;
        self.white -= other.white;
        self.colorless -= other.colorless;
        self.all -= other.all;

        for mana_to_remove in other.multi.iter().flatten() {
            assert!(self.multi.is_some());
            let mut found = false;
            let self_multi : Vec<Mana> = self.multi.as_ref().into_iter().flatten().filter_map(|m| {
                if !found && mana_to_remove.colors == m.colors {
                    found = true;
                    return None;
                }
                return Some(m.clone());
            }).collect();
            assert!(found);
            if !self_multi.is_empty() {
                self.multi = Some(self_multi);
            } else {
                self.multi = None;
            }
        }
    }

    /// Returns the converted mana cost.
    pub fn cmc(&self) -> u32 {
        return self.black
            + self.blue
            + self.green
            + self.red
            + self.white
            + self.colorless
            + self.all
            + match &self.multi {
                Some(vector) => vector.len() as u32,
                None => 0
            };
    }

    pub fn expanded(&self, other : &ManaPool) -> ManaPool {
        let mut pool = self.clone();
        pool.add_pool(other);
        return pool;
    }

    fn simple_clone(&self) -> Self {
        return ManaPool {
            black: self.black,
            blue: self.blue,
            green: self.green,
            red: self.red,
            white: self.white,
            colorless: self.colorless,
            all: self.all,
            multi: None
        };
    }

    pub fn can_also_pay_for(&self, already_spent: &ManaPool, additional_cost: &ManaPool) -> Option<ManaPool> {
        let total_cmc = already_spent.cmc() + additional_cost.cmc();
        if total_cmc > self.cmc() {
            return None;
        }

        let total = already_spent.expanded(additional_cost);
        if self.can_pay_for(&total) {
            return Some(total);
        }
        return None;
    }

    pub fn can_pay_for(&self, cost: &ManaPool) -> bool {
        if self.cmc() < cost.cmc() {
            return false;
        }

        let mut price = cost.clone();

        // costs that can be payed with any color can be considered colorless
        // for the purpose of this calculation..
        price.colorless += price.all;
        price.all = 0;

        if self.multi.is_some() && price.multi.is_some() {

            return true;

        } else if let Some(self_multi) = &self.multi {
            for colors in self_multi.iter().map(|mana| mana.colors.iter().collect::<Vec<Color>>()).multi_cartesian_product() {
                let mut me = self.simple_clone();
                colors.iter().for_each(|color| me.add_color(&color));
                if Self::check_simple_pools(me, price.simple_clone()) {
                    return true;
                }
            }
            return false;
        } else if let Some(price_multi) = &price.multi {
            for colors in price_multi.iter().map(|mana| mana.colors.iter().collect::<Vec<Color>>()).multi_cartesian_product() {
                let mut other = price.simple_clone();
                colors.iter().for_each(|color| other.add_color(&color));
                if Self::check_simple_pools(self.simple_clone(), other) {
                    return true;
                }
            }
            return false;
        } else {
            return Self::check_simple_pools(self.simple_clone(), price);
        }
    }

    fn check_simple_pools(mut pool: ManaPool, mut price: ManaPool) -> bool {

        fn pay(pool_color: &mut u32, cost_color: &mut u32) {
            if *pool_color >= *cost_color {
                *pool_color -= *cost_color;
                *cost_color = 0;
            } else {
                *cost_color -= *pool_color;
                *pool_color = 0;
            }
        }

        assert!(price.all == 0);
        assert!(pool.multi.is_none());
        assert!(price.multi.is_none());
        pay(&mut pool.black, &mut price.black);
        pay(&mut pool.blue, &mut price.blue);
        pay(&mut pool.green, &mut price.green);
        pay(&mut pool.red, &mut price.red);
        pay(&mut pool.white, &mut price.white);
        pay(&mut pool.colorless, &mut price.colorless);

        let monocolors_left_to_pay = price.black + price.blue + price.green + price.red + price.white;
        if monocolors_left_to_pay == 0 {
            // println!(" - no colors left to pay, checking colorless cost={} vs pool={}", price.colorless, pool.cmc());
            assert_eq!(price.cmc(), price.colorless);
            return pool.cmc() >= price.colorless;
        }
        if monocolors_left_to_pay <= pool.all {
            // println!(" - {} mono colored can be paid with {} any color", monocolors_left_to_pay, pool.all);
            return true;
        }

        // println!(" - {} mono colors left to play, only {} any available", monocolors_left_to_pay, pool.all);
        return false;
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mana_can_pay_for() {
        assert!(COLORLESS.can_pay_for(&COLORLESS));
        assert!(BLACK.can_pay_for(&BLACK));
        assert!(BLUE.can_pay_for(&BLUE));
        assert!(GREEN.can_pay_for(&GREEN));
        assert!(RED.can_pay_for(&RED));
        assert!(WHITE.can_pay_for(&WHITE));

        assert!(!COLORLESS.can_pay_for(&BLACK));
        assert!(!COLORLESS.can_pay_for(&BLUE));
        assert!(!COLORLESS.can_pay_for(&GREEN));
        assert!(!COLORLESS.can_pay_for(&RED));
        assert!(!COLORLESS.can_pay_for(&WHITE));

        assert!(BLACK.can_pay_for(&COLORLESS));
        assert!(BLUE.can_pay_for(&COLORLESS));
        assert!(GREEN.can_pay_for(&COLORLESS));
        assert!(RED.can_pay_for(&COLORLESS));
        assert!(WHITE.can_pay_for(&COLORLESS));

        assert!(ALL.can_pay_for(&BLACK));
        assert!(ALL.can_pay_for(&BLUE));
        assert!(ALL.can_pay_for(&GREEN));
        assert!(ALL.can_pay_for(&RED));
        assert!(ALL.can_pay_for(&WHITE));

        let selesnya = Mana::make_dual(Color::White, Color::Green);
        assert!(!selesnya.can_pay_for(&BLACK));
        assert!(!selesnya.can_pay_for(&BLUE));
        assert!(selesnya.can_pay_for(&GREEN));
        assert!(!selesnya.can_pay_for(&RED));
        assert!(selesnya.can_pay_for(&WHITE));
    }

    #[test]
    fn test_mana_can_pay_for_exactly() {
        assert!(COLORLESS.can_pay_for_exactly(&COLORLESS));
        assert!(BLACK.can_pay_for_exactly(&BLACK));
        assert!(BLUE.can_pay_for_exactly(&BLUE));
        assert!(GREEN.can_pay_for_exactly(&GREEN));
        assert!(RED.can_pay_for_exactly(&RED));
        assert!(WHITE.can_pay_for_exactly(&WHITE));

        assert!(!COLORLESS.can_pay_for_exactly(&BLACK));
        assert!(!COLORLESS.can_pay_for_exactly(&BLUE));
        assert!(!COLORLESS.can_pay_for_exactly(&GREEN));
        assert!(!COLORLESS.can_pay_for_exactly(&RED));
        assert!(!COLORLESS.can_pay_for_exactly(&WHITE));

        assert!(!BLACK.can_pay_for_exactly(&COLORLESS));
        assert!(!BLUE.can_pay_for_exactly(&COLORLESS));
        assert!(!GREEN.can_pay_for_exactly(&COLORLESS));
        assert!(!RED.can_pay_for_exactly(&COLORLESS));
        assert!(!WHITE.can_pay_for_exactly(&COLORLESS));

        assert!(!ALL.can_pay_for_exactly(&BLACK));
        assert!(!ALL.can_pay_for_exactly(&BLUE));
        assert!(!ALL.can_pay_for_exactly(&GREEN));
        assert!(!ALL.can_pay_for_exactly(&RED));
        assert!(!ALL.can_pay_for_exactly(&WHITE));

        let selesnya = Mana::make_dual(Color::White, Color::Green);
        assert!(!selesnya.can_pay_for_exactly(&BLACK));
        assert!(!selesnya.can_pay_for_exactly(&BLUE));
        assert!(!selesnya.can_pay_for_exactly(&GREEN));
        assert!(!selesnya.can_pay_for_exactly(&RED));
        assert!(!selesnya.can_pay_for_exactly(&WHITE));
    }

    #[test]
    fn test_mana_is_colorless() {
        assert!(COLORLESS.is_colorless());
        assert!(!BLACK.is_colorless());
        assert!(!BLUE.is_colorless());
        assert!(!GREEN.is_colorless());
        assert!(!RED.is_colorless());
        assert!(!WHITE.is_colorless());
    }

    #[test]
    fn test_mana_is_monocolored() {
        assert!(!COLORLESS.is_monocolor());
        assert!(BLACK.is_monocolor());
        assert!(BLUE.is_monocolor());
        assert!(GREEN.is_monocolor());
        assert!(RED.is_monocolor());
        assert!(WHITE.is_monocolor());
    }

    #[test]
    fn test_mana_contains() {
        assert!(ALL.contains(Color::Black));
        assert!(ALL.contains(Color::Blue));
        assert!(ALL.contains(Color::Green));
        assert!(ALL.contains(Color::Red));
        assert!(ALL.contains(Color::White));
        assert!(!WHITE.contains(Color::Black));
        assert!(!WHITE.contains(Color::Blue));
        assert!(!WHITE.contains(Color::Green));
        assert!(!WHITE.contains(Color::Red));
        assert!(WHITE.contains(Color::White));
    }

    #[test]
    fn test_manapool_display() {
        let one_of_each_color = ManaPool::new_from_sequence(&vec![BLACK, BLUE, GREEN, RED, WHITE, COLORLESS, ALL]);
        println!(" - one of each: {}", one_of_each_color);

        let many_black = ManaPool::new_from_sequence(&vec![BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK, BLACK]);
        println!(" - 8 black: {}", many_black);
    }

    #[test]
    fn test_manapool_can_pay_for() {
        let one_of_each_color = ManaPool::new_from_sequence(&vec![BLACK, BLUE, GREEN, RED, WHITE]);
        let green_plus_2 = ManaPool::new_from_sequence(&vec![COLORLESS, COLORLESS, GREEN]);
        let black_plus_1 = ManaPool::new_from_sequence(&vec![COLORLESS, BLACK]);
        let red_white_and_2 = ManaPool::new_from_sequence(&vec![COLORLESS, COLORLESS, RED, WHITE]);
        let colorless_times_5 = ManaPool::new_from_sequence(&vec![COLORLESS, COLORLESS, COLORLESS, COLORLESS, COLORLESS]);
        let any_x_2_plus_3 = ManaPool::new_from_sequence(&vec![ALL, ALL, COLORLESS, COLORLESS, COLORLESS]);

        assert!(one_of_each_color.can_pay_for(&green_plus_2));
        assert!(one_of_each_color.can_pay_for(&one_of_each_color));
        assert!(one_of_each_color.can_pay_for(&black_plus_1));
        assert!(one_of_each_color.can_pay_for(&red_white_and_2));
        assert!(one_of_each_color.can_pay_for(&colorless_times_5));

        assert!(!one_of_each_color.can_pay_for(&ManaPool::new_from_sequence(&vec![BLACK, BLACK])));
        assert!(!one_of_each_color.can_pay_for(&ManaPool::new_from_sequence(&vec![BLUE, BLUE])));
        assert!(!one_of_each_color.can_pay_for(&ManaPool::new_from_sequence(&vec![GREEN, GREEN])));
        assert!(!one_of_each_color.can_pay_for(&ManaPool::new_from_sequence(&vec![RED, RED])));
        assert!(!one_of_each_color.can_pay_for(&ManaPool::new_from_sequence(&vec![WHITE, WHITE])));

        assert!(any_x_2_plus_3.can_pay_for(&ManaPool::new_from_sequence(&vec![BLACK, BLACK])));
        assert!(any_x_2_plus_3.can_pay_for(&ManaPool::new_from_sequence(&vec![BLUE, BLUE])));
        assert!(any_x_2_plus_3.can_pay_for(&ManaPool::new_from_sequence(&vec![GREEN, GREEN])));
        assert!(any_x_2_plus_3.can_pay_for(&ManaPool::new_from_sequence(&vec![RED, RED])));
        assert!(any_x_2_plus_3.can_pay_for(&ManaPool::new_from_sequence(&vec![WHITE, WHITE])));

        let rakdos = ManaPool::new_from_sequence(&vec![Mana::make_dual(Color::Red, Color::Black)]);
        assert!(rakdos.can_pay_for(&ManaPool::new_from_sequence(&vec![BLACK])));
        assert!(!rakdos.can_pay_for(&ManaPool::new_from_sequence(&vec![BLUE])));
        assert!(!rakdos.can_pay_for(&ManaPool::new_from_sequence(&vec![GREEN])));
        assert!(rakdos.can_pay_for(&ManaPool::new_from_sequence(&vec![RED])));
        assert!(!rakdos.can_pay_for(&ManaPool::new_from_sequence(&vec![WHITE])));
    }

    #[test]
    fn test_manapool_remove_exact_pool() {
        let mut rakdos = ManaPool::new_from_sequence(&vec![Mana::make_dual(Color::Red, Color::Black)]);
        rakdos.remove_exact_pool(&rakdos.clone());
        assert!(rakdos.multi.is_none());
        assert_eq!(rakdos.cmc(), 0);

        let mut a_bit_of_each = ManaPool::new_from_sequence(&vec![BLACK, BLUE, GREEN, RED, WHITE, ALL, COLORLESS, COLORLESS]);
        a_bit_of_each.remove_exact_pool(&a_bit_of_each.clone());
        assert_eq!(a_bit_of_each.cmc(), 0);
    }

    #[test]
    fn test_manapool_perf() {
        let now = Instant::now();
        let pool = ManaPool::new_from_sequence(&vec![
            Mana::make_triple(Color::Black, Color::Blue, Color::Green),
            Mana::make_triple(Color::Black, Color::Blue, Color::Red),
            Mana::make_triple(Color::Black, Color::Blue, Color::White),
            Mana::make_triple(Color::Black, Color::Green, Color::Red),
            Mana::make_triple(Color::Black, Color::Green, Color::White),
            Mana::make_triple(Color::Black, Color::Red, Color::White),
            Mana::make_triple(Color::Blue, Color::Green, Color::Red),
            Mana::make_triple(Color::Blue, Color::Green, Color::White),
            Mana::make_triple(Color::Blue, Color::Red, Color::White),
            Mana::make_triple(Color::Green, Color::Red, Color::White)
        ]);
        let ur_dragon_cost = ManaPool::new_from_sequence(&vec![
            COLORLESS, COLORLESS, COLORLESS, COLORLESS,
            BLACK, BLUE, GREEN, RED, WHITE
        ]);
        assert!(pool.can_pay_for(&ur_dragon_cost));
        let elapsed = now.elapsed();
        println!("Elapsed (manapool): {:.2?}", elapsed);
    }

    #[test]
    fn test_manapool_new_from_string() {
        let pool_2x_all = ManaPool::new_from_string("{B/G/R/U/W}{B/G/R/U/W}").unwrap();
        assert_eq!(pool_2x_all.all, 2);
        assert_eq!(pool_2x_all.cmc(), 2);
        assert_eq!(pool_2x_all.black, 0);
        assert_eq!(pool_2x_all.blue, 0);
        assert_eq!(pool_2x_all.green, 0);
        assert_eq!(pool_2x_all.red, 0);
        assert_eq!(pool_2x_all.white, 0);
        assert_eq!(pool_2x_all.colorless, 0);
        assert!(pool_2x_all.multi.is_none());

        let ur_dragon_cost = ManaPool::new_from_string("{4}{B}{G}{R}{W}{U}").unwrap();
        assert_eq!(ur_dragon_cost.all, 0);
        assert_eq!(ur_dragon_cost.cmc(), 9);
        assert_eq!(ur_dragon_cost.black, 1);
        assert_eq!(ur_dragon_cost.blue, 1);
        assert_eq!(ur_dragon_cost.green, 1);
        assert_eq!(ur_dragon_cost.red, 1);
        assert_eq!(ur_dragon_cost.white, 1);
        assert_eq!(ur_dragon_cost.colorless, 4);
        assert!(ur_dragon_cost.multi.is_none());

        let obzedat_cost = ManaPool::new_from_string("{1}{B}{B}{W}{W}").unwrap();
        assert_eq!(obzedat_cost.all, 0);
        assert_eq!(obzedat_cost.cmc(), 5);
        assert_eq!(obzedat_cost.black, 2);
        assert_eq!(obzedat_cost.blue, 0);
        assert_eq!(obzedat_cost.green, 0);
        assert_eq!(obzedat_cost.red, 0);
        assert_eq!(obzedat_cost.white, 2);
        assert_eq!(obzedat_cost.colorless, 1);
        assert!(obzedat_cost.multi.is_none());

        let many_black = ManaPool::new_from_string("{B}{B}{B}{B}{B}{B}{B}{B}").unwrap();
        assert_eq!(many_black.all, 0);
        assert_eq!(many_black.cmc(), 8);
        assert_eq!(many_black.black, 8);
        assert_eq!(many_black.blue, 0);
        assert_eq!(many_black.green, 0);
        assert_eq!(many_black.red, 0);
        assert_eq!(many_black.white, 0);
        assert_eq!(many_black.colorless, 0);
        assert!(many_black.multi.is_none());

    }
}
