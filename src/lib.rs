use futures::future::join_all;
use scraper::{ElementRef, Html, Selector};

static ROW_COL_MAX_VAL: u16 = 50;

// TODO maybe possible to make a "merge" function that takes a PlayerH2H and merges it with another PlayerH2H with the same player name. Would need to ensure the order of things is maintained though...
struct PlayerH2H {
    player_name: String,
    ordered_head_to_heads: Vec<(u16, u16)>,
}

impl PlayerH2H {
    fn from_html_row(html_row: &ElementRef) -> Option<PlayerH2H> {
        let th_selector = Selector::parse("th").expect("Unable to make selector for 'th' element");
        let td_selector = Selector::parse("td").expect("Unable to make selector for 'td' element");

        let player_name = html_row
            .select(&th_selector)
            .map(|th| th.text().collect::<String>().trim().to_string())
            // The first entry in the vector is the row number, not the player name.
            // Grab the player name at the back.
            .collect::<Vec<String>>()
            .pop()?;

        let ordered_head_to_heads: Vec<(u16, u16)> = html_row
            .select(&td_selector)
            .map(|td| td.text().collect::<String>().trim().to_string())
            .filter(|h2h_string| h2h_string.chars().any(|c| c.is_ascii_digit()))
            .map(|h2h_string_w_values| {
                let mut chars = h2h_string_w_values.chars();
                let wins: u16 = chars
                    .next()
                    .expect("Could not find first character to map to win count")
                    as u16;
                let loses: u16 = chars
                    .next_back()
                    .expect("Could not find last character to map to loss count")
                    as u16;
                (wins, loses)
            })
            .collect();

        match !ordered_head_to_heads.is_empty() {
            true => Some(PlayerH2H {
                player_name,
                ordered_head_to_heads,
            }),
            false => None,
        }
    }
}

fn read_table_to_sub_player_h2h(h2h_table: ElementRef) -> Vec<PlayerH2H> {
    // The first table is not the h2h table, so skip to the second one
    let mut player_h2hs = vec![];
    let row_selector = Selector::parse("tr").unwrap();
    for row in h2h_table.select(&row_selector) {
        match PlayerH2H::from_html_row(&row) {
            Some(player_h2h) => player_h2hs.push(player_h2h),
            None => continue,
        }
    }

    player_h2hs
}

fn read_html_response_to_player_h2h(html_doc: Html) -> Vec<PlayerH2H> {
    let table_selector = Selector::parse("table").unwrap();
    let h2h_table = html_doc
        .select(&table_selector)
        .nth(1)
        .expect("Could not find h2h table as second table in html");

    read_table_to_sub_player_h2h(h2h_table)
}

fn convert_row_col_count_to_num_pages(html_doc: &Html) -> u16 {
    let row_col_count_selector = Selector::parse("div.input-group-addon.my-input-group-addon")
        .expect("Unable to parse row col selector");
    let row_col_count_string = html_doc
        .select(&row_col_count_selector)
        .nth(0)
        .expect("Failed to get HTML row count")
        .text()
        .collect::<String>();

    // Grab the final number in the "a to b of X rows" where we're looking for X.
    let row_col_count = row_col_count_string
        .split("of")
        .last()
        .expect("Failed to find 'of' in text, looking for 'a to b of x rows'")
        .trim()
        .split("\n")
        .next()
        .expect("Failed to find end-line in text")
        .trim()
        .parse::<u16>()
        .expect("Unable to convert text to u16");

    ((row_col_count as f32) / (ROW_COL_MAX_VAL as f32)).ceil() as u16
}

// TODO remove this
fn print_h2hs(league_id: &str, player_h2hs: Vec<PlayerH2H>) {
    println!("========================");
    println!("LEAGUE ID: {}", league_id);
    println!("Found {} players...", player_h2hs.len());
    // TODO need to generalize such that we get all the head to heads for each player, not just the first 50.
    for player_h2h in &player_h2hs {
        println!(
            "Player: {} has record with length: {:?}",
            player_h2h.player_name,
            player_h2h.ordered_head_to_heads.len()
        );
    }
    println!("========================");
}

async fn get_html_doc_from_url(url: &str) -> Html {
    let html_response = reqwest::get(url)
        .await
        .expect("Failed to receive HTTP response")
        .text()
        .await
        .expect("Failed to get HTML body from response");

    Html::parse_document(&html_response)
}

async fn process_league_to_player_h2hs(league_id: &str) -> Vec<PlayerH2H> {
    // https://braacket.com/league/{league_id}/head2head/{ranking_id}?rows={SIZE}&cols={SIZE}"
    //             + f"&page={r+1}&page_cols={c+1}&data=result&game_character=&country=&search=

    // TODO the league_id should be a ranking_id actually, and maybe make league_id configurable.
    let url = format!(
        "https://braacket.com/league/comelee/head2head/{}?rows={}&cols={}&page=1&page_cols=1&data=result&game_character=&country=&search=",
        league_id, ROW_COL_MAX_VAL, ROW_COL_MAX_VAL,
    );

    let html_doc = get_html_doc_from_url(&url).await;
    let num_pages = convert_row_col_count_to_num_pages(&html_doc);

    // Get first part of the table with this response and then iterate to get the rest.
    let player_h2hs = read_html_response_to_player_h2h(html_doc);
    print_h2hs(league_id, player_h2hs);
    for row in 1..num_pages {
        for col in 1..num_pages {
            let url = format!(
                "https://braacket.com/league/comelee/head2head/{}?rows={}&cols={}&page={}&page_cols={}&data=result&game_character=&country=&search=",
                league_id,
                ROW_COL_MAX_VAL,
                ROW_COL_MAX_VAL,
                row,
                col,
            );

            let html_doc = get_html_doc_from_url(&url).await;


        }
    }





    player_h2hs
}

pub async fn make_colley_ranking() -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let braacket_league_ids = [
        "F22828D5-E5E2-4A07-83A0-4D6FDCF7FB7C",
        "8982631B-07FF-4955-915C-CF8EC7AAAB72",
        "3A6E2789-CD62-4462-9F28-196FC8B05EA2",
        "B3B6A4C9-4C45-49B5-BC3E-97BFC07566E4",
        "1B2D2093-284F-4B5F-A1A7-F33814FCCBDE",
    ];

    let league_result_futures = braacket_league_ids
        .into_iter()
        .map(|league_id| process_league_to_player_h2hs(league_id))
        .collect::<Vec<_>>();

    let results = join_all(league_result_futures).await;
    println!("Finished processing in {}", start.elapsed().as_secs_f64());

    Ok(())
}
