use std::collections::HashMap;

use charton::prelude::*;
use polars::prelude::*;

use crate::charts::DarkTheme;

/// create a graph with the average rating of shows per genre
pub fn top_5_genre_average_rating_bargraph(df: &DataFrame) -> String {
    let genres_by_rating = df.columns(vec!["genre", "rating"]).unwrap();

    let (genres, ratings) = (
        genres_by_rating
            .get(0)
            .unwrap()
            .str()
            .unwrap()
            .iter()
            .map(|n| n.unwrap().to_string())
            .collect::<Vec<_>>(),
        genres_by_rating
            .get(1)
            .unwrap()
            .f64()
            .unwrap()
            .iter()
            .map(|n| n.unwrap())
            .collect::<Vec<_>>(),
    );

    let genres_and_ratings = genres.iter().zip(ratings).collect::<Vec<_>>();

    let mut rating_per_genre = HashMap::new();

    for (genre, rating) in genres_and_ratings {
        rating_per_genre
            .entry(genre)
            .or_insert(Vec::new())
            .push(rating);
    }

    let mut average_rating_per_genre = HashMap::new();

    for (genre, rating) in &rating_per_genre {
        let average_rating = rating.iter().sum::<f64>() / rating.iter().count() as f64;
        average_rating_per_genre.insert(genre.to_string(), average_rating);
    }

    let mut top_genres_by_rating = average_rating_per_genre.iter().collect::<Vec<_>>();
    top_genres_by_rating.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    let top_five_genres = top_genres_by_rating.iter().take(5).collect::<Vec<_>>();

    let genres = Column::new(
        "genres".into(),
        top_five_genres
            .iter()
            .map(|n| {
                n.0
                    // .svg and .png don't like "&" in labels and refuse to save to an image with these present
                    .replace("&", " ")
            })
            .collect::<Vec<_>>(),
    );

    let ratings = Column::new(
        "ratings".into(),
        top_five_genres
            .iter()
            .map(|n| n.1.clone())
            .collect::<Vec<_>>(),
    );

    let bargraph_df = DataFrame::new(vec![genres, ratings]).unwrap();

    let svg = Chart::build(&bargraph_df)
        .unwrap()
        .mark_point()
        .encode((x("genres"), y("ratings").with_zero(false)))
        .unwrap()
        .into_layered()
        .apply_dark_theme()
        .with_tick_label_font_size(11)
        .with_x_tick_label_angle(80.0)
        .apply_dark_theme()
        .with_title("top 5 genres by average ratings")
        .to_svg()
        .unwrap();
    svg
}
