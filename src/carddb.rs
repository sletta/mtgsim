use crate::mana;
use crate::card;

pub struct DB {
    pub entries : std::collections::HashMap<String, card::Card>,
    metadata : std::collections::HashMap<String, json::JsonValue>
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

fn parse_card_metadata(card : &mut card::Card, object : &json::JsonValue) -> Result<(), String> {

    fn parse_produces_tag(object : &json::JsonValue) -> Result<mana::Pool, String> {
        match object["produces"].as_str() {
            None => return Err("missing 'produces' property".to_string()),
            Some(mana_string) => {
                return mana::Pool::parse_cost(mana_string);
            }
        }
    }

    fn parse_count_tag(object : &json::JsonValue) -> Result<Vec<i32>, String> {
        match &object["count"] {
            json::JsonValue::Number(value) => {
                println!("that stupid value is: {:?}", value);
                return Err("god damnit...".to_string());
            },
            json::JsonValue::Array(array) => return Ok(array.iter().map(|j| j.as_i32().unwrap()).collect()),
            _ => return Err("invald 'count' tag".to_string())
        }
    }

    match object["ramp"].as_str() {
        None => { }, // this is ok
        Some("land-fetch") => {
            match &object["land_type"] {
                json::JsonValue::String(type_string) => card.ramp = Some(card::Ramp::LandFetch(vec![type_string.to_string()])),
                json::JsonValue::Array(type_strings) => {
                    let mut list : Vec<String> = Vec::new();
                    for i in type_strings {
                        list.push(i.to_string());
                    }
                    card.ramp = Some(card::Ramp::LandFetch(list))
                },
                _ => return Err("missing 'land_type' when parsing 'land-fetch'".to_string())
            }
        },
        Some("mana-producer") => {
            match parse_produces_tag(object) {
                Ok(ramp) => card.ramp = Some(card::Ramp::ManaProducer(ramp)),
                Err(failure) => return Err(failure)
            }
        }
        Some(_) => return Err("invalid ramp type".to_string())
    }

    match object["land"].as_str() {
        None => { }, // this is ok..
        Some("mana-producer") => {
            card.land_mana = Some(parse_produces_tag(object)?);
        },
        Some(_) => panic!("invalid 'land' type, {:?}", object)
    }

    if (object.has_key("draw")) {
        let count = parse_count_tag(object)
            .unwrap_or_else(|e| panic!("'count' failed, error={:?}, name={:?}, json={:?}",
                                       e,
                                       card.name,
                                       object));
        match object["draw"].as_str() {
            Some("one-shot") => card.draw = Some(card::Draw::OneShot(count)),
            Some("per-turn") => card.draw = Some(card::Draw::PerTurn(count)),
            _ => return Err("invalid key for 'draw'".to_string())
        }
    }

    return Ok(());
}

impl DB {

    pub fn new() -> Self {
        return Self {
            entries: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        }
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

        let mut entry = card::Card {
            name: json_object["name"].to_string(),
            cmc: json_object["cmc"].as_f32().expect("cmc is not a number!") as i32,
            mana_cost: match mana::Pool::parse_cost(&json_object["mana_cost"].to_string()) {
                Ok(pool) => Some(pool),
                Err(_) => None
            },
            type_string: type_line.clone(),
            types: card::parse_types(&type_line),
            produced_mana: parse_produced_mana(&json_object["produced_mana"]),
            enters_tapped: parse_enters_tapped(&name, &json_object["oracle_text"].to_string()),
            ramp: None,
            draw: None,
            land_mana: None
        };

        match self.metadata.get(name) {
            Some(metadata) => {
                match parse_card_metadata(&mut entry, metadata) {
                    Err(failure) => {
                        panic!("failed to parse metadata for {:?}, error: {:?}, json: {:?}",
                               name,
                               failure,
                               metadata);
                    },
                    Ok(_) => { }
                }
            },
            None => println!(" - no metadata...")
        }

        self.entries.insert(name.to_string(), entry);

        return self.entries.get(name);
    }

    pub fn load_metadata(&mut self, name : &str) {
        println!("loading metadata from: '{:?}'", name);
        let metadata_json = std::fs::read_to_string(&name).unwrap_or_else(|e| panic!("failed to load file, file={:?}, error={:?}", name, e));
        let json = json::parse(&metadata_json).unwrap_or_else(|e| panic!("failed to parse json, file_name={:?}, error={:?}", name, e));

        assert!(json.is_array());

        for object in json.members() {
            let name = object["name"].to_string().to_lowercase();
            self.metadata.insert(name, object.clone());
        }
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

        "mana-producer"
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

