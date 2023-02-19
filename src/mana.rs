#[derive(Eq, PartialEq, Clone)]
pub struct Mana {
    black : bool,
    blue : bool,
    green : bool,
    red : bool,
    white : bool
}

pub const COLORLESS : Mana  = Mana { black: false, blue: false, green: false, red: false, white: false };
pub const BLACK : Mana      = Mana { black: true,  blue: false, green: false, red: false, white: false };
pub const BLUE : Mana       = Mana { black: false, blue: true,  green: false, red: false, white: false };
pub const GREEN : Mana      = Mana { black: false, blue: false, green: true,  red: false, white: false };
pub const RED : Mana        = Mana { black: false, blue: false, green: false, red: true,  white: false };
pub const WHITE : Mana      = Mana { black: false, blue: false, green: false, red: false, white: true  };

#[derive(Clone)]
pub struct Pool {
    sequence: Vec<Mana>
}

impl std::fmt::Debug for Mana {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut colors = Vec::new();
        if self.black {
            colors.push("B");
        }
        if self.blue {
            colors.push("U");
        }
        if self.green {
            colors.push("G");
        }
        if self.red {
            colors.push("R");
        }
        if self.white {
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

    pub fn set_from_string(&mut self, value : &str) {
        match value {
            "C" => {
                // do nothing..
            }
            "B" => self.black = true,
            "U" => self.blue = true,
            "G" => self.green = true,
            "R" => self.red = true,
            "W" => self.white = true,
            _ => panic!("bad input, '{}'", value)
        }
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
            write!(f, "{{{:?}}}", i).expect("formatting failed!");
        }
        return Ok(());
    }
}

impl Pool {

    pub fn new() -> Self {
        return Self { sequence: Vec::new() };
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
}
