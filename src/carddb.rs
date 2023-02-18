use crate::mana;
use crate::card;

pub struct DB {
    entries : std::collections::HashMap<String, card::Card>,
}

fn name_to_file(name : &str) -> String {
    return format!("cards.db/{}.json", name).to_owned();
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

        let entry = card::Card {
            name: json_object["name"].to_string(),
            cmc: json_object["cmc"].as_f32().expect("cmc is not a number!") as i32,
            mana_cost: mana::Cost::parse(&json_object["mana_cost"].to_string()).unwrap_or_else(|e| panic!("failed to parse mana_cost, '{:?}', error={:?}", json_object["mana_cost"], e))
        };

        self.entries.insert(name.to_string(), entry);

        return self.entries.get(name);
    }
}
