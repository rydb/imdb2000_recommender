use std::collections::HashMap;

use charton::prelude::*;
use polars::prelude::*;

use crate::charts::DarkTheme;

pub fn top_five_genres_piechart(df: &DataFrame) -> String {
    let shows_by_genre = df.columns(vec!["genre", "id"]).unwrap();

    let (show_genres, _show_ids) = (
        shows_by_genre.get(0).unwrap(),
        shows_by_genre.get(1).unwrap(),
    );

    let show_genre_instances = show_genres
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap().to_owned())
        .collect::<Vec<_>>();

    let mut genres_count_unordered = HashMap::new();

    for genre in show_genre_instances {
        *genres_count_unordered.entry(genre).or_insert(0) += 1;
    }

    let mut genres_count_ordered = genres_count_unordered
        .iter()
        .map(|n| (n.0.clone(), n.1.clone()))
        .collect::<Vec<_>>();
    genres_count_ordered.sort_by(|(_, a), (_, b)| b.cmp(a));

    println!(
        "most popular genre: {:#?}",
        (
            genres_count_ordered.get(0).unwrap().0.clone(),
            genres_count_ordered.get(0).unwrap().1
        )
    );

    let top_five_genres = genres_count_ordered
        .clone()
        .iter()
        .map(|n| format!("{}({})", n.0.clone(), n.1))
        .take(5)
        .collect::<Vec<_>>();

    let top_five_genre_show_instances = genres_count_ordered
        .clone()
        .iter()
        .map(|n| n.1)
        .take(5)
        .collect::<Vec<_>>();

    let pie_chart_category_count = Column::new("value".into(), top_five_genre_show_instances);

    let pie_chart_categories = Column::new("category".into(), top_five_genres);

    let pie_chart_df =
        DataFrame::new(vec![pie_chart_categories, pie_chart_category_count]).unwrap();

    let chart = Chart::build(&pie_chart_df)
        .unwrap()
        .mark_arc()
        .encode((theta("value"), color("category")))
        .unwrap();
    let layered_chart = LayeredChart::new();

    let svg = layered_chart
        .with_title("5 most common genres of the IMDB 2000")
        .apply_dark_theme()
        .add_layer(chart)
        .to_svg()
        .unwrap();

    println!("FINISHED GENERATING PIECHART");
    return svg;
}
