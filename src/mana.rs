use itertools::Itertools;

const BLACK_FLAG : u32   = 0x01;
const BLUE_FLAG : u32    = 0x02;
const GREEN_FLAG : u32   = 0x04;
const RED_FLAG : u32     = 0x08;
const WHITE_FLAG : u32   = 0x10;

pub const COLORLESS : Mana  = Mana { flags: 0x0 };
pub const BLACK : Mana      = Mana { flags: BLACK_FLAG };
pub const BLUE : Mana       = Mana { flags: BLUE_FLAG };
pub const GREEN : Mana      = Mana { flags: GREEN_FLAG };
pub const RED : Mana        = Mana { flags: RED_FLAG };
pub const WHITE : Mana      = Mana { flags: WHITE_FLAG };

#[derive(Eq, PartialEq, Clone)]
pub struct Mana {
    flags : u32
}

#[derive(Clone)]
pub struct Pool {
    sequence: Vec<Mana>
}

impl std::fmt::Debug for Mana {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut colors = Vec::new();
        if self.flags & BLACK_FLAG != 0 {
            colors.push("B");
        }
        if self.flags & BLUE_FLAG != 0 {
            colors.push("U");
        }
        if self.flags & GREEN_FLAG != 0 {
            colors.push("G");
        }
        if self.flags & RED_FLAG != 0 {
            colors.push("R");
        }
        if self.flags & WHITE_FLAG != 0 {
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

impl std::fmt::Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.sequence.len() == 0 {
            return write!(f, "n/a");
        }
        let colorless_cost = self.sequence.iter().filter(|&mana| *mana == COLORLESS).count();
        if colorless_cost > 0 {
            write!(f, "{{{}}}", colorless_cost).expect("formatting failed");
        }
        for i in self.sequence.iter().filter(|&mana| *mana != COLORLESS) {
            write!(f, "{:?}", i).expect("formatting failed!");
        }
        return Ok(());
    }
}

impl Mana {
    pub fn new() -> Self {
        return COLORLESS.clone();
    }

    pub fn set_from_string(&mut self, value : &str) {
        match value {
            "C" => {
                // do nothing..
            }
            "B" => self.flags |= BLACK_FLAG,
            "U" => self.flags |= BLUE_FLAG,
            "G" => self.flags |= GREEN_FLAG,
            "R" => self.flags |= RED_FLAG,
            "W" => self.flags |= WHITE_FLAG,
            _ => panic!("bad input, '{}'", value)
        }
    }

    pub fn is_colorless(&self) -> bool {
        return self.flags == 0;
    }

    pub fn is_monocolor(&self) -> bool {
        return self.flags == BLACK_FLAG
            || self.flags == BLUE_FLAG
            || self.flags == GREEN_FLAG
            || self.flags == RED_FLAG
            || self.flags == WHITE_FLAG;
    }

    pub fn can_pay_for(&self, other : &Mana) -> bool {
        return other.flags == 0 || ((self.flags & other.flags) != 0);
    }

    pub fn can_pay_for_exactly(&self, other : &Mana) -> bool {
        return self.flags == other.flags && (self.is_colorless() || self.is_monocolor());
    }
}

impl Pool {

    pub fn new() -> Self {
        return Self { sequence: Vec::new() };
    }

    pub fn converted_mana_cost(&self) -> u32 {
        return self.sequence.len() as u32;
    }

    pub fn parse_cost(cost : &str) -> Result<Self, String> {
        let re = regex::Regex::new(r"([0-9BCGRUW/]+)").expect("failed to crate manacost reggexp");
        let mut colors : Vec<Mana> = Vec::new();
        for cap in re.find_iter(cost) {
            let value = cap.as_str();
            match value {
                "C" => colors.push(COLORLESS),
                "B" => colors.push(BLACK),
                "U" => colors.push(BLUE),
                "G" => colors.push(GREEN),
                "R" => colors.push(RED),
                "W" => colors.push(WHITE),
                _ => {
                    let count = value.parse::<u32>().unwrap_or_else(|e| panic!("failed to parse mana value! error={:?}, value='{:?}'", e, value));
                    for _i in 0..count {
                        colors.push(COLORLESS);
                    }
                }
            }
        }

        let mana_cost = Pool { sequence: colors };
        return Ok(mana_cost);
    }

    pub fn can_pay_for(&self, other : &Pool) -> bool {
        if other.converted_mana_cost() > self.converted_mana_cost() {
            return false;
        }

        let mut source = self.sequence.clone();
        let mut cost = other.sequence.clone();

        // To cut down on the number of permutations we need to look at we can
        // start out by subtracting all perfect matches right away. If we're
        // asking being asked for a green mana and we have a source providing
        // exactly green mana, then we can simply extract that cost from both
        // the source and the cost and cut down our iteration down below
        // quite a bit.
        let mut i = 0;
        while i < cost.len() {
            let mut exact_match : i32 = -1;
            for j in 0..source.len() {
                if source[j].can_pay_for_exactly(&cost[i]) {
                    exact_match = j as i32;
                    break;
                }
            }
            if exact_match >= 0 {
                source.remove(exact_match as usize);
                cost.remove(i);
            } else {
                i += 1;
            }
        }

        // Generate all permutations of the mana sources. If compatible, we'll
        // then eventually land on a permutation of mana sources that can pay
        // for the specificed cost.
        let mut perms = 0;
        for perm in source.iter().permutations(source.len()) {
            perms = perms + 1;
            let mut accepted = true;
            for i in 0..cost.len() {
                if !perm[i].can_pay_for(&cost[i]) {
                    accepted = false;
                    continue;
                }
            }
            if accepted {
                return true;
            }
        }

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

        let all = Mana { flags: BLACK_FLAG | BLUE_FLAG | GREEN_FLAG | RED_FLAG | WHITE_FLAG };
        assert!(all.can_pay_for(&BLACK));
        assert!(all.can_pay_for(&BLUE));
        assert!(all.can_pay_for(&GREEN));
        assert!(all.can_pay_for(&RED));
        assert!(all.can_pay_for(&WHITE));

        let selesnya = Mana { flags: WHITE_FLAG | GREEN_FLAG };
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

        let all = Mana { flags: BLACK_FLAG | BLUE_FLAG | GREEN_FLAG | RED_FLAG | WHITE_FLAG };
        assert!(!all.can_pay_for_exactly(&BLACK));
        assert!(!all.can_pay_for_exactly(&BLUE));
        assert!(!all.can_pay_for_exactly(&GREEN));
        assert!(!all.can_pay_for_exactly(&RED));
        assert!(!all.can_pay_for_exactly(&WHITE));

        let selesnya = Mana { flags: WHITE_FLAG | GREEN_FLAG };
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
    fn test_pool_can_pay_for() {
        let one_of_each_color = Pool { sequence: vec![ BLACK, BLUE, GREEN, RED, WHITE ] };
        let green_plus_2 = Pool { sequence: vec![ COLORLESS, COLORLESS, GREEN ] };
        let black_plus_1 = Pool { sequence: vec![ COLORLESS, BLACK ] };
        let red_white_and_2 = Pool { sequence: vec![ COLORLESS, COLORLESS, RED, WHITE ] };
        let colorless_times_5 = Pool { sequence: vec![ COLORLESS, COLORLESS, COLORLESS, COLORLESS, COLORLESS ] };

        assert!(one_of_each_color.can_pay_for(&green_plus_2));
        assert!(one_of_each_color.can_pay_for(&one_of_each_color));
        assert!(one_of_each_color.can_pay_for(&black_plus_1));
        assert!(one_of_each_color.can_pay_for(&red_white_and_2));
        assert!(one_of_each_color.can_pay_for(&colorless_times_5));

        assert!(!one_of_each_color.can_pay_for(&Pool { sequence: vec![ BLACK, BLACK ] }));
        assert!(!one_of_each_color.can_pay_for(&Pool { sequence: vec![ BLUE, BLUE ] }));
        assert!(!one_of_each_color.can_pay_for(&Pool { sequence: vec![ GREEN, GREEN ] }));
        assert!(!one_of_each_color.can_pay_for(&Pool { sequence: vec![ RED, RED ] }));
        assert!(!one_of_each_color.can_pay_for(&Pool { sequence: vec![ WHITE, WHITE ] }));

        let rakdos = Pool { sequence: vec![ Mana { flags: BLACK_FLAG | RED_FLAG } ] };
        assert!(rakdos.can_pay_for(&Pool { sequence: vec![ BLACK] }));
        assert!(!rakdos.can_pay_for(&Pool { sequence: vec![ BLUE ] }));
        assert!(!rakdos.can_pay_for(&Pool { sequence: vec![ GREEN ] }));
        assert!(rakdos.can_pay_for(&Pool { sequence: vec![ RED ] }));
        assert!(!rakdos.can_pay_for(&Pool { sequence: vec![ WHITE ] }));

        let two_of_each_color = Pool { sequence: vec![ BLACK, BLUE, GREEN, RED, WHITE, BLACK, BLUE, GREEN, RED, WHITE ]};
        let freaky_cost_1 = Pool { sequence: vec![ GREEN, GREEN, RED, RED, WHITE, WHITE, BLUE, COLORLESS, COLORLESS, BLUE ]};
        assert!(two_of_each_color.can_pay_for(&freaky_cost_1));
    }
}
