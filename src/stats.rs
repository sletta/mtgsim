use crate::game;

fn average(sum: u32, count: usize) -> f32 {
    return sum as f32 / count as f32;
}

fn show_commander_stats(stats: &Vec<game::GameStats>, _settings: &game::Settings) {
    let not_played = stats.iter().filter(|s| s.turn_commander_played == 0).count();
    let played = stats.len() - not_played;
    let commander_average_turn = average(stats.iter().map(|s| s.turn_commander_played).sum(), played);

    println!("Commander arrives on turn ........: {:.1} (avg)", commander_average_turn);
    println!("games Commander didn't arrive ....: {:.2}% ({})", 100.0 * not_played as f32 / stats.len() as f32, not_played);
}

fn show_draw_stats(stats: &Vec<game::GameStats>, settings: &game::Settings) {
    let total_turn_count = stats.len() * (settings.turn_count as usize);
    let draw_curve = average(stats.iter().map(|s| s.turns_stats.iter()).flatten().map(|s| s.cards_drawn).sum(), total_turn_count);
    println!("cards/round average ..............: {:.2}", draw_curve);

    let out_of_cards = stats.iter().filter(|s| s.out_of_cards).count();
    println!("games library ran out of cards ...: {:.2}% ({})", 100.0 * out_of_cards as f32 / stats.len() as f32, out_of_cards);
}

fn show_ram_stats(stats: &Vec<game::GameStats>, settings: &game::Settings) {

    let ramp_curve: f32 = stats
        .iter()
        .map(|s| s.turns_stats[settings.turn_count as usize - 1].mana_available)
        .sum::<u32>() as f32 / (stats.len() as f32 * settings.turn_count as f32);

    println!("mana increase / turn (ramp) ......: {:.2} mana / turn", ramp_curve);
}

fn show_performance_stats(stats: &Vec<game::GameStats>, settings: &game::Settings) {
    let mut cmd_played_turns: Vec<u32> = vec![0; (settings.turn_count + 1) as usize];
    stats.iter().for_each(|s| cmd_played_turns[s.turn_commander_played as usize] += 1);
    println!();
    println!("                                      Turn breakdown / Deck Performance");
    println!();
    println!("           --------- cards ---------   ----- lands -----   ---------- mana ---------   --- commander ---");
    println!("  Turn      drawn   played  in-hand     played  cheated     total    spent    ratio     %-played  (abs)");
    for i in 0..settings.turn_count {
        let index = i as usize;

        let cards_drawn = average(stats.iter().map(|s| s.turns_stats[index].cards_drawn).sum(), stats.len());
        let cards_played = average(stats.iter().map(|s| s.turns_stats[index].cards_played).sum(), stats.len());
        let cards_in_hand = average(stats.iter().map(|s| s.turns_stats[index].cards_in_hand).sum(), stats.len());

        let lands_played = average(stats.iter().map(|s| s.turns_stats[index].lands_played).sum(), stats.len());
        let lands_cheated = average(stats.iter().map(|s| s.turns_stats[index].lands_cheated).sum(), stats.len());

        let mana_available = average(stats.iter().map(|s| s.turns_stats[index].mana_available).sum(), stats.len());
        let mana_spent = average(stats.iter().map(|s| s.turns_stats[index].mana_spent).sum(), stats.len());

        let cmd_played: f32 = 100.0 * cmd_played_turns[index+1] as f32 / stats.len() as f32;

        let padding = if i > 8 { "" } else { " " };
        println!("  #{}:{}     {:5.2}    {:5.2}    {:5.2}      {:5.2}    {:5.2}      {:5.2}    {:5.2}    {:5.2}      {:5.1}%   {:>4}",
                i + 1, padding,
                cards_drawn, cards_played, cards_in_hand,
                lands_played, lands_cheated,
                mana_available, mana_spent, mana_spent / mana_available,
                cmd_played, cmd_played_turns[index+1]);
    }
}

pub fn show_statistics(stats: &Vec<game::GameStats>, settings: &game::Settings) {
    show_performance_stats(stats, settings);

    println!();

    show_commander_stats(stats, settings);
    show_draw_stats(stats, settings);
    show_ram_stats(stats, settings);

    println!();
    println!("games simulated ..................: {}", stats.len());
    println!("turns per game ...................: {}", settings.turn_count);
}
