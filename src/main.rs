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

    show_statistics(&stats, &settings);
}

fn average(sum: u32, count: usize) -> f32 {
    return sum as f32 / count as f32;
}

fn show_statistics(stats: &Vec<game::GameStats>, settings: &game::Settings) {
    println!("Played {} games of {} turns each:", stats.len(), settings.turn_count);

    let last_turn_index : usize = settings.turn_count as usize - 1;

    let commander_average_turn = average(stats.iter().map(|s| s.turn_commander_played).sum(), stats.len());
    println!(" - commander arrives on turn: {:.2}", commander_average_turn);

    let total_turn_count = stats.len() * (settings.turn_count as usize);

    let draw_curve = average(stats.iter().map(|s| s.turns_stats.iter()).flatten().map(|s| s.cards_drawn).sum(), total_turn_count);
    println!(" - deck speed: {:.2} cards / round (average)", draw_curve);

    let mut mana_available_curve : f32 = 0.0;
    let mut mana_spending_curve : f32 = 0.0;
    for s in stats {
        let last_turn = &s.turns_stats[last_turn_index];
        mana_available_curve += last_turn.mana_available as f32;
        mana_spending_curve += last_turn.mana_spent as f32;
    }
    mana_available_curve /= total_turn_count as f32;
    mana_spending_curve /= total_turn_count as f32;

    println!(" - ramp curve: {:.2} mana / turns (average)", mana_available_curve);
    println!(" - mana spending curve: {:.2} mana / turns (average), effectiveness: {}", mana_spending_curve, mana_spending_curve / mana_available_curve);
    // println!(" - {:.2} mana / turns (average)", mana_available_curve / total_turn_count as f32);

    for round in 0..settings.turn_count {
        let i : usize = round as usize;

        let lands_played = average(stats.iter().map(|s| s.turns_stats[i].lands_played).sum(), stats.len());
        let lands_cheated = average(stats.iter().map(|s| s.turns_stats[i].lands_cheated).sum(), stats.len());
        let cards_drawn = average(stats.iter().map(|s| s.turns_stats[i].cards_drawn).sum(), stats.len());
        let cards_played = average(stats.iter().map(|s| s.turns_stats[i].cards_played).sum(), stats.len());
        let cards_in_hand = average(stats.iter().map(|s| s.turns_stats[i].cards_in_hand).sum(), stats.len());
        let mana_available = average(stats.iter().map(|s| s.turns_stats[i].mana_available).sum(), stats.len());
        let mana_spent = average(stats.iter().map(|s| s.turns_stats[i].mana_spent).sum(), stats.len());

        let mut mana_effectiveness : f32 = 0.0;
        for s in stats {
            let turn_stats = &s.turns_stats[i];
            mana_effectiveness += turn_stats.mana_spent as f32 / turn_stats.mana_available as f32;
        }
        mana_effectiveness /= stats.len() as f32;

        println!(" - turn #{:2}: cards(drawn={:.1}, played={:.1} in-hand={:.1}), lands(played={:.2}, cheated={:.1}), mana(spent={:.2} total={:.2}, effectiveness={:.2})",
                round + 1,
                cards_drawn,
                cards_played,
                cards_in_hand,
                lands_played,
                lands_cheated,
                mana_spent,
                mana_available,
                mana_effectiveness);
    }
}
