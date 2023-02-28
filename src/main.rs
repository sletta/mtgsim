mod carddb;
mod mana;
mod card;
mod game;
mod zone;

use std::io::BufRead;
use regex::Regex;
use clap::Parser;

#[derive(Debug)]
struct DeckListEntry {
    count : u32,
    name : String,
}

fn read_deck_list(file_name : &str) -> Result<Vec<DeckListEntry>, String> {
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
    return Ok(deck_list);
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    metadata : String,

    #[arg(short, long)]
    decklist : String,

    #[arg(short, long)]
    commander : String,
}

fn main() {

    let args = Arguments::parse();

    let deck_list = read_deck_list(&args.decklist).unwrap();

    let mut db = carddb::DB::new();
    db.load_metadata(&args.metadata);

    let bullshit = vec![1, 2, 3, 4, 5, 6, 8];
    bullshit.iter().for_each(|n| println!("{}", n));

    let mut found_commander = false;

    deck_list.iter().for_each(|e| match db.load(&e.name) {
        Some(card_data) => {
            if card_data.name == args.commander {
                assert_eq!(e.count, 1);
                found_commander = true;
            }
        },
        None => panic!("failed to load card: {}", e.name)
    });

    if !found_commander {
        panic!("commander {} was not found in the decklist...", args.commander);
    }

    let mut game = game::Game::new();

    deck_list.iter().for_each(|e| {
        let card_data = &db.entries[&e.name];
        let card = card::Card::new(&card_data);
        if card_data.name == args.commander {
            assert_eq!(e.count, 1);
            game.command.add(card);
        } else {
            for _ in 0..e.count {
                game.library.add(card.clone())
            }
        }
    });

    // game.setup();

    let settings = game::Settings {
        turn_count: 2,
        draw_card_on_turn_one: true
    };
    game.play(&settings);

    // game.dump();

    // for (entry in )
    // deck_list.iter().for_each(|e| println!("{:?}", e) );

    // let deck_list = read_deck_list(&deck_list_name.unwrap()).unwrap();

    // let mut db = carddb::DB::new();

    // db.load_metadata(&card_metadata_name.unwrap());

    // for entry in &deck_list {
    //     db.load(&entry.name).expect("loading card failed!");
    // }

    // let mut game = game::Game::new();
    // for entry in &deck_list {
    //     let card : &card::Card = &db.entries[&entry.name];
    //     for _i in 0..entry.count {
    //         game.library.add(card.clone());
    //     }
    // }

    // game.setup();

}
