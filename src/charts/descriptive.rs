use charton::prelude::*;
use polars::prelude::*;
use roberta::show_prediction::RobertaModel;

use crate::charts::DarkTheme;

/// Visualize the theme confidence of a given show
pub async fn generate_descriptive_theme_confidence_graph<const ENTRIES: usize>(
    roberta: RobertaModel,
    show: &str,
) -> String {
    let confidence_ratings = roberta
        .prompt::<ENTRIES>(format!("{show} is a show about <mask>."))
        .await;

    let themes = Column::new("themes".into(), confidence_ratings.clone().map(|n| n.value));
    let confidence = Column::new(
        "confidence".into(),
        confidence_ratings.map(|n| n.confidence),
    );

    let df = DataFrame::new(vec![themes, confidence].into()).unwrap();

    let svg = Chart::build(&df)
        .unwrap()
        .mark_point()
        .encode((
            x("themes"),
            y("confidence").with_zero(false),
            color("themes"),
        ))
        .unwrap()
        .into_layered()
        .apply_dark_theme()
        .with_title(format!("descriptions of themes of: {show}"))
        .to_svg()
        .unwrap();

    svg
    // .save("./assets/charts/descriptive.svg")
    // .unwrap();
}
