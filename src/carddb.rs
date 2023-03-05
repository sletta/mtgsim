use crate::mana;
use crate::card;

pub struct DB {
    pub entries : std::collections::HashMap<String, card::CardData>,
    metadata : std::collections::HashMap<String, json::JsonValue>,
    pub verbose : bool
}

fn name_to_file(name : &str) -> String {
    return format!("cards.db/{}.json", name.to_owned().replace("/", "_")).to_owned();
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
        Some(_) => return true,
        None => return false
    }
}

fn parse_mana_pool(object: &json::object::Object, property: &str) -> Result<mana::ManaPool, String> {
    match object[property].as_str() {
        Some(string) => Ok(mana::ManaPool::new_from_string(string)?),
        None => Err("invalid 'mana::produce' value".to_string())
    }
}

fn parse_cost(object : &json::JsonValue) -> Result<card::Cost, String> {
    fn parse_cost_string(string : &str) -> Result<card::Cost, String> {
        match string {
            "tap" => Ok(card::Cost::Tap),
            "tap-sacrifice" => Ok(card::Cost::TapSacrifice),
            "none" => Ok(card::Cost::None),
            _ => Err("invalid 'cost' string!".to_string())
        }
    }

    match &object["cost"] {
        json::JsonValue::Short(txt) => parse_cost_string(txt),
        json::JsonValue::String(txt) => parse_cost_string(txt),
        json::JsonValue::Object(cost_object) => {
            match cost_object["type"].as_str() {
                Some("tap") => Ok(card::Cost::Tap),
                Some("tap-sacrifice") => Ok(card::Cost::TapSacrifice),
                Some("none") => Ok(card::Cost::None),
                Some("tap-mana-sacrifice") => Ok(card::Cost::TapManaSacrifice(parse_mana_pool(cost_object, "mana")?)),
                _ => Err("invalid 'cost::type' value...".to_string()),
            }
        },
        json::JsonValue::Null => Ok(card::Cost::None),
        _ => Err("invalid 'cost' value".to_string())
    }
}

fn parse_trigger(object : &json::JsonValue) -> Result<card::Trigger, String> {
    fn parse_trigger_string(string : &str) -> Result<card::Trigger, String> {
        match string {
            "activated" => Ok(card::Trigger::Activated),
            "upkeep" => Ok(card::Trigger::Upkeep),
            "cast" => Ok(card::Trigger::Cast),
            _ => Err("invalid 'trigger' string".to_string())
        }
    }
    match &object["trigger"] {
        json::JsonValue::Short(txt) => parse_trigger_string(txt),
        json::JsonValue::String(txt) => parse_trigger_string(txt),
        _ => Err("invalid 'trigger' value!".to_string())
    }
}

fn parse_effect_land_fetch(object : &json::object::Object) -> Result<card::Effect, String> {

    fn parse_string_or_array(value : &json::JsonValue) -> Result<Vec<String>, String> {
        let mut v : Vec<String> = Vec::new();
        match &value {
            json::JsonValue::Null => (),
            json::JsonValue::Short(txt) => v.push(txt.to_string()),
            json::JsonValue::String(txt) => v.push(txt.to_string()),
            json::JsonValue::Array(array) => for i in array { v.push(i.to_string()); },
            _ => return Err("invalid 'to-hand' or 'to-battlefield' value...".to_string())
        }
        return Ok(v);
    }

    return Ok(card::Effect::FetchLand {
        to_hand: parse_string_or_array(&object["to-hand"])?,
        to_battlefield: parse_string_or_array(&object["to-battlefield"])?
    });
}

fn parse_effect_draw(object: &json::object::Object) -> Result<card::Effect, String> {
    let json_count = &object["count"];
    if json_count.is_array() {
        let mut draw_ratios: Vec<u32> = Vec::new();
        for value in json_count.members() {
            draw_ratios.push(value.as_u32().unwrap());
        }
        return Ok(card::Effect::Draw(draw_ratios));
    } else if let Some(single_value) = json_count.as_u32() {
        return Ok(card::Effect::Draw(vec![single_value]));
    }
    return Err("failed to parse 'draw' effect!".to_string());
}

fn parse_effect_mana(object : &json::object::Object) -> Result<card::Effect, String> {
    match object["produce"].as_str() {
        Some(string) => Ok(card::Effect::ProduceMana(mana::ManaPool::new_from_string(string)?)),
        None => Err("invalid 'mana::produce' value".to_string())
    }
}

fn parse_effect_land_limit(object: &json::object::Object) -> Result<card::Effect, String> {
    if let Some(increase) = &object["increase"].as_u32() {
        return Ok(card::Effect::LandLimit(increase.clone()));
    }
    return Err("invalid 'increase' in land-limit".to_string())
}

fn parse_effect(object : &json::JsonValue) -> Result<card::Effect, String> {
    match &object["effect"] {
        json::JsonValue::Object(effect_object) => {
            match effect_object["type"].as_str() {
                Some("mana") => Ok(card::Effect::ProduceMana(parse_mana_pool(effect_object, "produce")?)),
                Some("land-fetch") => parse_effect_land_fetch(effect_object),
                Some("draw") => parse_effect_draw(effect_object),
                Some("land-limit") => parse_effect_land_limit(effect_object),
                _ => Err("invalid 'effect::type' string".to_string())
            }
        },
        _ => Err("invalid 'effect' value!".to_string())
    }
}

fn parse_availability(object : &json::JsonValue) -> f32 {
    if let Some(availability) = object["availability"].as_f32() {
        return availability;
    }
    return 1.0;
}

fn parse_ability(object : &json::JsonValue) -> Result<card::Ability, String> {
    return Ok(card::Ability {
        trigger: parse_trigger(object)?,
        cost : parse_cost(object)?,
        effect : parse_effect(object)?,
        availability : parse_availability(object)
    });
}

fn parse_card_metadata(card : &mut card::CardData, object : &json::JsonValue) -> Result<(), String> {

    if object.has_key("abilities") {
        let json_abilities = &object["abilities"];
        if !json_abilities.is_array() {
            return Err("'abilities' must be an array".to_string());
        }
        let mut abilities : Vec<card::Ability> = Vec::new();
        for ability in json_abilities.members() {
            abilities.push(parse_ability(ability)?);
        }
        card.abilities = Some(abilities);
    } else {
        card.abilities = Some(vec![parse_ability(object)?]);
    }

    return Ok(());
}

impl DB {

    pub fn new() -> Self {
        return Self {
            entries: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            verbose: false,
        }
    }

    pub fn load(&mut self, name : &str) -> Option<&card::CardData> {
        if self.verbose {
            println!("loading: {}", name);
        }
        let file_name = name_to_file(name);
        let contents;

        if !std::path::Path::new(&file_name).exists() {
            let url = format!("https://api.scryfall.com/cards/named?exact=\"{}\"", name);
            println!(" -> downloading: {}...", url);
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

        let mut entry = card::CardData {
            name: json_object["name"].to_string(),
            cmc: json_object["cmc"].as_f32().expect("cmc is not a number!") as u32,
            mana_cost: match mana::ManaPool::new_from_string(&json_object["mana_cost"].to_string()) {
                Ok(pool) => Some(pool),
                Err(_) => None
            },
            type_string: type_line.clone(),
            types: card::parse_types(&type_line),
            produced_mana: parse_produced_mana(&json_object["produced_mana"]),
            enters_tapped: parse_enters_tapped(&name, &json_object["oracle_text"].to_string()),
            abilities: None
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
            None => println!("Missing metadata for: '{}'", entry.name)
        }

        if self.verbose {
            println!("DB entry: '{}'", name);
            println!("{:?}", entry);
        }

        self.entries.insert(name.to_string(), entry);

        return self.entries.get(name);
    }

    pub fn load_metadata(&mut self, name : &str) {
        if self.verbose {
            println!("loading metadata from: '{:?}'", name);
        }
        let metadata_json = std::fs::read_to_string(&name).unwrap_or_else(|e| panic!("failed to load file, file={:?}, error={:?}", name, e));
        let json = json::parse(&metadata_json).unwrap_or_else(|e| panic!("failed to parse json, file_name={:?}, error={:?}", name, e));

        assert!(json.is_array());

        for object in json.members() {
            let name = object["name"].to_string().to_lowercase();
            self.metadata.insert(name, object.clone());
        }
    }

    pub fn alias(&self, name : &str) -> Option<String> {
        let json = self.metadata.get(name)?;
        return Some(json["alias"].as_str()?.to_string().to_lowercase());
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

