mod carddb;
mod mana;
mod card;
mod game;
mod zone;

use std::io::BufRead;
use regex::Regex;

#[derive(Debug)]
struct DeckListEntry {
    count : u32,
    name : String,
}

fn read_deck_list(file_name : &str) -> Option<Vec<DeckListEntry>> {
    let mut  deck_list : Vec<DeckListEntry> = Vec::new();
    let file = std::fs::File::open(file_name).unwrap();
    let lines = std::io::BufReader::new(file).lines();
    let re = Regex::new(r"^(\d+)x\s+([\w\s',\-]+)").unwrap();
    for maybe_line in lines {
        let line = maybe_line.unwrap().clone();
        let captures = re.captures(&line).unwrap();
        // println!(" - {:?}", captures);
        // if let Some(captures) = re.captures(line) {
        let count = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let name = captures.get(2).unwrap().as_str();
        deck_list.push(DeckListEntry {
            count: count,
            name: name.trim().to_lowercase()
        } );
    }
    return Some(deck_list);
}

fn main() {

    let args: Vec<String> = std::env::args().collect();
    let mut deck_list_name : Option<String> = None;
    let mut card_metadata_name : Option<String> = None;

    let mut skip_next = false;
    for i in 0..args.len() {
        if skip_next {
            skip_next = false;
            continue;
        }
        let arg = &args[i];
        if arg == "--deck-list" && i < args.len() - 1 {
            deck_list_name = Some(args[i+1].clone());
            skip_next = true;
        } else if arg == "--card-metadata" && i < args.len() - 1 {
            card_metadata_name = Some(args[i+1].clone());
            skip_next = true;
        }
    }

    if deck_list_name.is_none() {
        println!("missing --deck-list [text-file] argument!");
        return;
    }

    // if card_metadata_name.is_none() {
    //     println("missing --card-metadata [json-file] argument!");
    //     return;
    // }

    let deck_list = read_deck_list(&deck_list_name.unwrap()).unwrap();

    let mut db = carddb::DB::new();

    db.load_metadata(&card_metadata_name.unwrap());

    for entry in &deck_list {
        db.load(&entry.name).expect("loading card failed!");
    }

    let mut game = game::Game::new();
    for entry in &deck_list {
        let card : &card::Card = &db.entries[&entry.name];
        for _i in 0..entry.count {
            game.library.add(card.clone());
        }
    }

    game.setup();

}
