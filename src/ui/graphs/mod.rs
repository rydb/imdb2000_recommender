use dioxus::prelude::*;
use futures_util::StreamExt;
use itertools::Itertools;
use polars::{
    frame::DataFrame,
    io::SerReader,
    prelude::{CsvReader, IntoVec},
};
use rig::{
    agent::Agent,
    completion::Prompt,
    providers::deepseek::{self},
};
use roberta::show_prediction::RobertaModel;
use std::io::Cursor;

use base64::{Engine as _, engine::general_purpose};

use crate::{
    charts::{
        DATABASE_AS_STR,
        bargraph::top_5_genre_average_rating_bargraph,
        descriptive::generate_descriptive_theme_confidence_graph,
        donut_chart::top_ten_shows_count_by_country_donut,
        pie_chart::top_five_genres_piechart,
        predictive::{ShowOpinion, generate_perdictive_show_appreciation_scores_graph},
    },
    ui::{
        Agents, AgentsCollection, HorizontalScroll, ShowRecommendation, Signal,
        SingleShowRecommendation, Styles, active_or, err_rsx,
    },
};

pub mod descriptive;
pub use descriptive::*;

pub mod predictive;
pub use predictive::*;

#[derive(Clone, PartialEq)]
pub enum GraphTabChoice {
    Statistical,
    Ai,
}
#[derive(Clone, PartialEq)]
pub enum AiGraphTabChoice {
    Predictive,
    Descriptive,
}

const ENTRIES: usize = 4;

lazy_static::lazy_static! {
    static ref EMPTY_ALT: [String; 4] = ["".to_owned(), "".to_owned(), "".to_owned(), "".to_owned()];
}

#[derive(Clone)]
pub struct UserShowOpinions(Signal<Result<[ShowOpinion; ENTRIES], String>>);

impl Default for UserShowOpinions {
    fn default() -> Self {
        Self(Signal::new(Err(
            "predictive user show opinions not initialized".into(),
        )))
    }
}

pub async fn recommend_similar_shows<const N: usize>(
    recommended_show: &ShowRecommendation,
    four_similiar_show_recommender_agent: &Agent<deepseek::CompletionModel>,
) -> Result<[String; N], String> {
    let prompt = format!(
        "
    [SHOW]
    {}
    [SHOW]
    ",
        recommended_show.show
    );
    let result = four_similiar_show_recommender_agent
        .prompt(prompt)
        .await
        .map_err(|err| err.to_string())?;
    let split = result
        .splitn(ENTRIES, ",")
        .map(|n| n.to_string())
        .collect::<Vec<_>>();
    let similar_shows_result = split.as_array::<N>().cloned().ok_or(format!(
        "unable to split shows in set of {ENTRIES} show(s) {result}, split {:#?}",
        split
    ))?;

    Ok(similar_shows_result)
}

/// Component for AI graphs
#[component]
pub fn AiGraphs(
    recommended_show: Signal<ShowRecommendation>,
    agents: Signal<AgentsCollection>,
) -> Element {
    let mut graph_choices = use_signal(|| AiGraphTabChoice::Descriptive);

    let similar_shows = use_resource(move || async move {
        println!("GENERATING NEW RECOMMENDED SHOWS");

        recommend_similar_shows::<ENTRIES>(
            &*recommended_show.read(),
            &agents.read().four_similiar_show_recommender_agent,
        )
        .await
    });

    rsx! {
        div {
            button {
                class: active_or(*graph_choices.read() == AiGraphTabChoice::Predictive, Styles::selected, ""),
                onmouseup: move |_event| {
                    *graph_choices.write() = AiGraphTabChoice::Predictive
                },
                "Predictive Graph"
            }
            button {
                class: active_or(*graph_choices.read() == AiGraphTabChoice::Descriptive, Styles::selected, ""),
                onmouseup: move |_event| {
                    *graph_choices.write() = AiGraphTabChoice::Descriptive
                },
                "Descriptive Graph"
            }
        }

        div {
            hidden: *graph_choices.read() != AiGraphTabChoice::Predictive,
            flex_direction: "row",
            PerdictiveGraph { recommended_show, agents, similar_shows }

        }

        div {
            flex_direction: "row",
            hidden: *graph_choices.read() != AiGraphTabChoice::Descriptive,
            DescriptiveGraph { recommended_show, agents }
        }
    }
}

/// Components for dataset stats graphs
#[component]
pub fn StatisticalGraphs() -> Element {
    let df = use_context::<IMDB2000Database>();
    let pie_chart = use_memo(move || {
        let df = &*df.0.read();
        let svg = top_five_genres_piechart(df);
        load_chart_as_url(&svg)
    });

    let donut_chart = use_memo(move || {
        let df = &*df.0.read();
        let svg = top_ten_shows_count_by_country_donut(df);
        load_chart_as_url(&svg)
    });

    let bar_chart = use_memo(move || {
        let df = &*df.0.read();
        let svg = top_5_genre_average_rating_bargraph(df);
        load_chart_as_url(&svg)
    });

    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            align_items: "baseline",
            h1 {
                u {
                    "Dataset Trivia"
                }

            }
            h1 {
                ":  "
            }
            h4 {
                {
                    " (Scroll or use scrollbar view)"
                }
            }
        }
        HorizontalScroll {
            div {
                flex_direction: "row",
                background: "white",

                img {
                    width: "500px",
                    height: "400px",
                    src: pie_chart,
                }
                img {
                    width: "500px",
                    height: "400px",
                    src: donut_chart
                }
                img {
                    width: "500px",
                    height: "400px",
                    src: bar_chart
                }
            }
        }
    }
}
const SVG_URL: &'static str = "data:image/svg+xml;base64,";

/// Takes a graph as a string, and encodes it to be readable by an html url
pub fn load_chart_as_url(file_as_str: &str) -> String {
    let base64_string = general_purpose::STANDARD.encode(file_as_str);
    return SVG_URL.to_owned() + &base64_string;
}

/// Database saved into a polars dataframe
#[derive(Clone)]
pub struct IMDB2000Database(pub Signal<DataFrame>);

/// Menu for selecting different graphs
#[component]
pub fn GraphOptions() -> Element {
    let agents = use_context::<Agents>();
    let show_recommendation = use_context::<SingleShowRecommendation>();

    let _df = use_context_provider(|| {
        IMDB2000Database(Signal::new(
            CsvReader::new(Cursor::new(DATABASE_AS_STR))
                .finish()
                .unwrap(),
        ))
    });

    let mut graph_choices = use_signal(|| GraphTabChoice::Statistical);

    let show_ai_graphs = use_memo(move || {
        let Ok(recommended_show) = show_recommendation.0.cloned() else {
            return err_rsx(
                "Awaiting show recommendation.. Please enter a show in the prompt textbox and hit \"enter\" with it hovered over",
            );
        };
        let Ok(agents) = agents.0.cloned() else {
            return err_rsx("loading agents..");
        };

        rsx!(AiGraphs {
            recommended_show,
            agents
        })
    });

    rsx! {
        div {
            class: Styles::card,
            width: "500px",
            div {
                b {
                    "Display new graphs by generating new prompts"
                }
            }
            div {
                flex_direction: "row",
                button {
                    class: active_or(*graph_choices.read() == GraphTabChoice::Ai, Styles::selected, ""),
                    onclick: move |_event| {
                        *graph_choices.write() = GraphTabChoice::Ai;
                    },

                    "AI Graphs(Predictive and Descriptive)"
                }
                button {
                    class: active_or(*graph_choices.read() == GraphTabChoice::Statistical, Styles::selected, ""),
                    onclick: move |_event| {
                        *graph_choices.write() = GraphTabChoice::Statistical
                    },
                    "Dataset Trivia"
                }
            }

            div {
                hidden: *graph_choices.read() != GraphTabChoice::Ai,
                {show_ai_graphs}
            }
            div {
                hidden: *graph_choices.read() != GraphTabChoice::Statistical,
                StatisticalGraphs {}
            }
            ,
        }


    }
}
