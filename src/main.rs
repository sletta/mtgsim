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
    let re = Regex::new(r"^(\d+)x\s+([\w\s',\-\\/]+)").unwrap();
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

fn parse_mulligan(txt : &Option<String>) -> game::MulliganType {
    match txt {
        Some(text) => match text.as_str() {
            "3-lands" => game::MulliganType::ThreeLands,
            "none" => game::MulliganType::None,
            _ => panic!("invalid mulligan type specified, only '3-lands' and 'nona' are available..")
        },
        None => game::MulliganType::None
    }
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

    #[arg(short, long, default_value_t = 10)]
    rounds : u32,

    #[arg(short, long, default_value_t = 1)]
    games : u32,

    #[arg(long, default_value_t = false)]
    verbose_db : bool,

    #[arg(long, default_value_t = false)]
    verbose_game : bool,

    #[arg(long)]
    mulligan : Option<String>
}

fn main() {

    let args = Arguments::parse();

    let mut db = carddb::DB::new();
    if args.verbose_db {
        db.verbose = true;
    }
    db.load_metadata(&args.metadata);

    let mut deck_list = read_deck_list(&args.decklist).unwrap();
    for entry in deck_list.iter_mut() {
        match db.alias(&entry.name) {
            Some(alias) => entry.name = alias,
            None => ()
        }
    }

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

    let mut stem_game = game::Game::new();
    if args.verbose_game {
        stem_game.verbose = true;
    }

    deck_list.iter().for_each(|e| {
        let card_data = &db.entries[&e.name];
        let card = card::Card::new(&card_data);
        if card_data.name == args.commander {
            assert_eq!(e.count, 1);
            stem_game.command.add(card);
        } else {
            for _ in 0..e.count {
                stem_game.library.add(card.clone())
            }
        }
    });

    let settings = game::Settings {
        turn_count: args.rounds,
        draw_card_on_turn_one: true,
        mulligan : parse_mulligan(&args.mulligan),
    };

    let mut stats : Vec<game::GameStats> = Vec::new();

    for _ in 0..args.games {
        let mut game = stem_game.clone();
        game.play(&settings);
        stats.push(game.game_stats.clone());
    }

    for s in stats {
        println!("{:?}", s);
    }

}
