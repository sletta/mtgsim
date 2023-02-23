use crate::mana;
use crate::card;

pub struct DB {
    pub entries : std::collections::HashMap<String, card::Card>,
}

fn name_to_file(name : &str) -> String {
    return format!("cards.db/{}.json", name).to_owned();
}

fn parse_produced_mana(value : &json::JsonValue) -> Option<mana::Mana> {
    if value.is_array() {
        let mut colors = mana::Mana::new();
        for i in 0..value.len() {
            colors.set_from_string(value[i].to_string().as_str());
        }
        return Some(colors);
    }
    return None;
}

fn parse_enters_tapped(name : &str, oracle_text : &str) -> bool {
    let pattern = format!("{} enters the battlefield tapped.", name);
    match oracle_text.to_lowercase().find(pattern.as_str()) {
        Some(_i) => return true,
        None => return false
    }
}


fn parse_card_metadata(mut card : &card::Card, object : &json::JsonValue) {
    let ramp_string = &object["ramp"];
    if ramp_string.is_string() {
        println!("{:?} is a ramp card, type={:?}", card.name, ramp_string);
    } else if !ramp_string.is_empty() {
        panic!("bad 'ramp' string for '{:?}'", object);
    }

    let draw_string = &object["draw"];
    if draw_string.is_string() {
        println!("{:?} is a draw card, type={:?}", card.name, draw_string);
    } else if !draw_string.is_empty() {
        panic!("bad 'draw' string for '{:?}'", object);
    }
}


impl DB {

    pub fn new() -> Self {
        return Self { entries: std::collections::HashMap::new() }
    }

    pub fn load(&mut self, name : &str) -> Option<&card::Card> {
        println!("loading: {}", name);
        let file_name = name_to_file(name);
        let mut contents : Option<String> = None;

        if !std::path::Path::new(&file_name).exists() {
            let url = format!("https://api.scryfall.com/cards/named?exact=\"{}\"", name);
            let response = reqwest::blocking::get(&url).unwrap_or_else(|e| panic!("download failed: url={:?}, error={:?}", url, e));
            let text = response.text().unwrap();
            std::fs::write(&file_name, text.as_bytes()).expect("failed to write downloaded file...");
            contents = Some(text);
            assert!(contents != None);
        } else {
            contents = Some(std::fs::read_to_string(&file_name).unwrap_or_else(|e| panic!("failed to load file, name={:?}, file={:?}, error={:?}", name, file_name, e)));
        }
        assert!(contents != None);

        let json_object = json::parse(&contents.unwrap()).unwrap_or_else(|e| panic!("failed to parse json, file_name={:?}, error={:?}", file_name, e));
        assert!(json_object.is_object());

        let type_line = json_object["type_line"].to_string();

        let entry = card::Card {
            name: json_object["name"].to_string(),
            cmc: json_object["cmc"].as_f32().expect("cmc is not a number!") as i32,
            mana_cost: mana::Pool::parse_cost(&json_object["mana_cost"].to_string()).unwrap_or_else(|e| panic!("failed to parse mana_cost, '{:?}', error={:?}", json_object["mana_cost"], e)),
            type_string: type_line.clone(),
            types: card::parse_types(&type_line),
            produced_mana: parse_produced_mana(&json_object["produced_mana"]),
            enters_tapped: parse_enters_tapped(&name, &json_object["oracle_text"].to_string()),
        };

        self.entries.insert(name.to_string(), entry);

        return self.entries.get(name);
    }

    pub fn load_metadata(&mut self, name : &str) {
        println!("loading metadata from: '{:?}", name);
        let metadata_json = std::fs::read_to_string(&name).unwrap_or_else(|e| panic!("failed to load file, file={:?}, error={:?}", name, e));
        let json = json::parse(&metadata_json).unwrap_or_else(|e| panic!("failed to parse json, file_name={:?}, error={:?}", name, e));

        assert!(json.is_array());

        for (key, value) in self.entries.iter_mut() {
            for object in json.members() {
                let name = object["name"].to_string();
                if key.eq(&name.to_lowercase()) {
                    parse_card_metadata(value, object);
                }
            }
        }

        //     let &card = self.entries.get(name.to_lowercase()).unwrap_or_else()
        // }
    }
}

/*
    The different classes and properties:

    * "name" : String
        -> Card name, exact match

    * "enters_tapped" : bool
        -> self explanatory.

    * "ramp" : String
        -> used ot indicate ramp spells, will be one of the following:

        "mana-rock"
            -> can be tapped for mana, Sol Ring, etc. Requires
               additional properties:

            "produces" : String
                -> Mana pool string, "{U/B}" and the like

        "land-fetch" -> fetches lands in library, also includes sub
                        properties:

            * "land_type" : String

    * "draw" : String
        -> the category that draws us cards, will be one of the
           following:

        "on-cast"
            -> triggers when cast / etb

        "per-turn"
            -> triggers on every turn

        -> In addition, "draw" type spells needs the following property
           set:

        "count" : Array<int>
            -> an array outlining the various amounts of cards the spell
               can draw. While a "Phyerxian Arena" is an "per-turn" spell
               that consistently draws 1 additional card, cards
               like "Armorcraft Judge", have much more complex
               interaction and thus the array outlines the probabilities

*/

