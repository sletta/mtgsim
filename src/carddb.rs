use crate::mana;
use crate::card;

pub struct DB {
    entries : std::collections::HashMap<String, card::Card>,
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
}
