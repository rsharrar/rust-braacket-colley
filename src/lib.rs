use anyhow::anyhow;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

// TODO maybe possible to make a "merge" function that takes a PlayerH2H and merges it with another PlayerH2H with the same player name. Would need to ensure the order of things is maintained though...
struct PlayerH2H {
    player_name: String,
    ordered_head_to_heads: Vec<(u16, u16)>,
}

impl PlayerH2H {
    fn from_html_chunk(html_chunk: &str) -> anyhow::Result<PlayerH2H> {
        let player_regex = Regex::new(r"title='([^']+)'").expect("Couldn't parse player regex");
        let win_loss_regex =
            Regex::new(r"\t(\d+) - (\d+)[\t\n]").expect("Couldn't parse win/loss regex");
        match player_regex.captures(html_chunk) {
            Some(player_capture) => {
                let player_name = player_capture.get(1).ok_or(anyhow!("Did not capture player name"))?.as_str().to_owned();

                let mut ordered_head_to_heads = Vec::new();
                for w_l_capture in win_loss_regex.captures_iter(html_chunk) {
                    let w_l_str: Vec<&str> =
                        w_l_capture.get(0).ok_or(anyhow!("Failed to capture win / loss record for player: {}", player_name))?.as_str().split(" - ").collect();
                    let wins: u16 = w_l_str[0].trim().parse()?;
                    let loses: u16 = w_l_str[1].trim().parse()?;
                    ordered_head_to_heads.push((wins, loses));
                }

                Ok(
                    PlayerH2H {
                        player_name,
                        ordered_head_to_heads,
                    }
                )
            }
            None => {
                Err(anyhow!("Could not capture player name"))
            }
        }
    }
}

fn read_html_response_to_player_h2h(html_response: &str) -> Vec<PlayerH2H> {
    let h2h_table_html = html_response
        .split("<tbody>")
        .last()
        .expect("Expected html body to contain <tbody>");
    let h2h_chunks: Vec<&str> = h2h_table_html.split("<a href=").collect();

    h2h_chunks
        .par_iter()
        .map(|chunk| PlayerH2H::from_html_chunk(chunk))
        .filter_map(|player_h2h| {
            match player_h2h {
                Ok(player_h2h) => {Some(player_h2h)}
                Err(e) => {println!("Unable to process player in table with error: {}", e); None}
            }
        })
        .collect()
}

pub async fn process_html_to_internal() -> anyhow::Result<()> {
    let url = "https://braacket.com/league/comelee/head2head/F22828D5-E5E2-4A07-83A0-4D6FDCF7FB7C?rows=50&cols=50&page=1&page_cols=1&data=result&game_character=&country=&search=";
    let body = reqwest::get(url).await?.text().await?;

    let start = std::time::Instant::now();
    let player_h2hs = read_html_response_to_player_h2h(&body);

    // TODO need to generalize such that we get all the head to heads for each player, not just the first 50.
    for player_h2h in player_h2hs {
        println!("Player: {} has record: {:?}", player_h2h.player_name, player_h2h.ordered_head_to_heads);
    }

    println!("Finished processing in {}", start.elapsed().as_secs_f64());
    Ok(())
}
