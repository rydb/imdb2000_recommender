use charton::prelude::*;
use polars::prelude::*;

use crate::charts::DarkTheme;

#[derive(Clone)]
pub struct ShowOpinion {
    pub show: String,
    pub perdiction: String,
    pub confidence: f32,
}

/// accuracy metrics from 0-100%
#[derive(Clone)]
pub struct Accuracy {
    pub roberta: f32,
    pub deepseek: f32,
}

// accuracy metric for calculations
fn calculate_brier_score(predictions: Vec<(bool, f32)>, actuals: Vec<(bool, f32)>) -> f32 {
    predictions
        .iter()
        .zip(actuals.iter())
        .map(|(pred, actual)| {
            let prob = if pred.0 { pred.1 } else { 1.0 - pred.1 };
            let actual_binary = if actual.0 { 1.0 } else { 0.0 };
            (prob - actual_binary).powi(2)
        })
        .sum::<f32>()
        / predictions.len() as f32
}

// Perdict the shows that a user would like, compared to the actual shows the given user liked.
pub fn generate_perdictive_show_appreciation_scores_graph<const ENTRIES: usize>(
    original_show: &String,
    show_like_predictions_deepseek: &[ShowOpinion; ENTRIES],
    show_like_perdictions_roberta: &[ShowOpinion; ENTRIES],
    show_like_perdictions_user: &Option<[ShowOpinion; ENTRIES]>,
) -> (String, DataFrame, Option<Accuracy>) {
    let mut shows_deepseek = Vec::new();
    let mut shows_roberta = Vec::new();
    let mut shows_user = Vec::new();

    let mut confidences_deepseek = Vec::new();
    let mut confidences_roberta = Vec::new();
    let mut confidences_user = Vec::new();

    let mut like_perdictions_deepseek = Vec::new();
    let mut like_perdictions_roberta = Vec::new();
    let mut like_perdictions_user = Vec::new();

    let mut brier_deepseek = Vec::new();
    let mut brier_roberta = Vec::new();
    let mut brier_user_actual = Vec::new();

    for show in show_like_predictions_deepseek.clone() {
        shows_deepseek.push(show.show);
        confidences_deepseek.push(show.confidence);
        like_perdictions_deepseek.push(show.perdiction.clone() + "_deepseek");

        let mut yes_no = false;

        // assume y is short for yes
        if show.perdiction.starts_with("y") {
            yes_no = true;
        };
        brier_deepseek.push((yes_no, show.confidence))
    }

    for show in show_like_perdictions_roberta.clone() {
        shows_roberta.push(show.show);
        confidences_roberta.push(show.confidence);
        like_perdictions_roberta.push(show.perdiction.clone() + "_roberta");

        let mut yes_no = false;

        // assume y is short for yes
        if show.perdiction.starts_with("y") {
            yes_no = true;
        };
        brier_roberta.push((yes_no, show.confidence))
    }
    if let Some(user_opinions) = show_like_perdictions_user.clone() {
        for show in user_opinions {
            shows_user.push(show.show);
            confidences_user.push(show.confidence);
            like_perdictions_user.push(show.perdiction.clone() + "_user");

            let mut yes_no = false;

            // assume y is short for yes
            if show.perdiction.starts_with("y") {
                yes_no = true;
            };
            brier_user_actual.push((yes_no, show.confidence))
        }
    }

    let mut accuracy = None;

    if show_like_perdictions_user.is_some() {
        let deepseek_brier_score = calculate_brier_score(brier_deepseek, brier_user_actual.clone());
        let roberta_brier_score = calculate_brier_score(brier_roberta, brier_user_actual);

        accuracy = Some(Accuracy {
            deepseek: deepseek_brier_score,
            roberta: roberta_brier_score,
        })
    }

    let max_length = 30;

    let shows = Vec::new()
        .iter()
        .chain(shows_deepseek.iter())
        .chain(shows_roberta.iter())
        .chain(shows_user.iter())
        // remove characters from show names that break serialization
        .map(|n| {
            n.replace("&", "and")
                .replace(",", " ")
                .replace("%", " ")
                .replace("#", " ")
                .replace("{", " ")
                .replace("}", " ")
                .replace("@", " ")
                .replace(".", " ")
                .replace(">", " ")
                .replace("<", " ")
                .replace("'", " ")
                .replace("^", " ")
                .replace("(", " ")
                .replace(")", " ")
        })
        // iterate through shows, and if they're too long to visually see, cut them off.
        .map(|mut n| {
            if n.len() >= max_length {
                n = n.chars().take(max_length).collect::<String>() + "-"
            }
            n
        })
        .collect::<Vec<_>>();

    let confidences = Vec::new()
        .iter()
        .chain(confidences_deepseek.iter())
        .chain(confidences_roberta.iter())
        .chain(confidences_user.iter())
        .map(|n| n.clone())
        .collect::<Vec<_>>();

    let like_perdictions = Vec::new()
        .iter()
        .chain(like_perdictions_deepseek.iter())
        .chain(like_perdictions_roberta.iter())
        .chain(like_perdictions_user.iter())
        .map(|n| n.clone())
        .collect::<Vec<_>>();

    let df = DataFrame::new(vec![
        Column::new("shows".into(), shows),
        Column::new("confidence".into(), confidences),
        Column::new("like_perdictions".into(), like_perdictions),
    ])
    .unwrap();

    let svg = Chart::build(&df)
        .unwrap()
        .mark_point()
        .encode((
            x("shows"),
            y("confidence").with_normalize(true).with_stack(false),
            color("like_perdictions"),
        ))
        .unwrap()
        .into_layered()
        .with_x_tick_label_angle(45.0)
        .with_title(format!(
            "Interest perdiction for shows like {}",
            original_show
        ))
        .apply_dark_theme()
        .to_svg()
        .unwrap();
    // .save("./assets/charts/predictive.svg")
    // .unwrap();
    (svg, df, accuracy)
}
