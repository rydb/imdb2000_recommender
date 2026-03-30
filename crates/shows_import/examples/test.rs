use polars::prelude::*;
use shows_import::show::ShowRecord;

#[derive(Debug)]
pub struct ShowDataset {
    pub shows: Vec<ShowRecord>,
    pub genre_vocab: Vec<String>,
}

pub fn generate_show_records(file_path: &str) -> Vec<ShowRecord> {
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(file_path.into()))
        .unwrap()
        .finish()
        .unwrap();

    let ids = df
        .column("id")
        .unwrap()
        .i64()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();
    let titles = df
        .column("title")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();
    let original_titles = df
        .column("original_title")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();
    // set overview to ??? for entries without an overview because training still needs these entries.
    let overviews = df
        .column("overview")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap_or("???"))
        .collect::<Vec<_>>();
    let premiere_dates = df
        .column("premiere_date")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();
    let genres = df
        .column("premiere_date")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();
    let country_of_origins = df
        .column("country_origin")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .collect::<Vec<_>>();
    let original_languages = df
        .column("original_language")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();
    let ratings = df
        .column("rating")
        .unwrap()
        .f64()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();
    let votes = df
        .column("votes")
        .unwrap()
        .i64()
        .unwrap()
        .iter()
        .map(|n| n.unwrap())
        .collect::<Vec<_>>();

    let mut show_records = Vec::new();

    for i in 0..ids.iter().len() - 1 {
        let record = ShowRecord {
            id: ids[i].to_string(),
            title: titles[i].to_string(),
            original_title: original_titles[i].to_string(),
            overview: overviews[i].to_string(),
            premiere_date: Some(premiere_dates[i].to_string()),
            genre: genres[i].to_string(),
            country_origin: country_of_origins[i].map(|n| n.to_string()),
            original_language: original_languages[i].to_string(),
            rating: Some(ratings[i]),
            votes: Some(votes[i]),
        };
        show_records.push(record);
    }
    show_records
}

pub fn main() {
    let shows = generate_show_records("assets/datasets/top_rated_2000webseries.csv");
    println!("{:#?}", shows)
}
