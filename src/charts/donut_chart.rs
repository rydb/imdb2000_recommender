use std::collections::HashMap;

use charton::prelude::*;
use polars::prelude::*;

use crate::charts::DarkTheme;

/// show shows by country
pub fn top_ten_shows_count_by_country_donut(df: &DataFrame) -> String {
    let countries_of_shows = df
        .column("country_origin")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(|n| n.unwrap_or("unknown").to_owned())
        .collect::<Vec<_>>();

    let mut show_country_count = HashMap::new();

    for country in countries_of_shows {
        *show_country_count.entry(country).or_insert(0) += 1;
    }

    let mut show_country_count_sorted = show_country_count.iter().collect::<Vec<_>>();
    show_country_count_sorted.sort_by(|(_, a), (_, b)| b.cmp(a));

    let donut_df = DataFrame::new(vec![
        Column::new(
            "category".into(),
            show_country_count_sorted
                .iter()
                .take(10)
                .map(|n| n.0.clone())
                .collect::<Vec<_>>(),
        ),
        Column::new(
            "value".into(),
            show_country_count_sorted
                .iter()
                .take(10)
                .map(|n| n.1.clone())
                .collect::<Vec<_>>(),
        ),
    ])
    .unwrap();

    let chart = Chart::build(&donut_df)
        .unwrap()
        .mark_arc()
        .encode((theta("value"), color("category")))
        .unwrap()
        .with_inner_radius_ratio(0.5);
    let layered_chart = LayeredChart::new();

    let svg = layered_chart
        .with_title("Country distribution of top 10 shows")
        .add_layer(chart)
        .apply_dark_theme()
        .to_svg()
        .unwrap();
    // .save("./assets/charts/donut_chart.svg")
    // .unwrap();

    svg
}
