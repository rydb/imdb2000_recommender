use polars::frame::DataFrame;

use crate::charts::predictive::Accuracy;

use super::*;

/// generate perdictions for interests in show with deepseek
pub async fn perdict_show_interests_deepseek<const N: usize>(
    original_request: &String,
    shows: &[String; N],
    agent: &Agent<deepseek::CompletionModel>,
) -> Result<[ShowOpinion; N], String> {
    let fmt_str = shows.iter().into_vec().join(", ");

    println!("similar shows to predict: {fmt_str}");

    let mut shows_as_string = "".to_owned();

    for show in shows {
        shows_as_string += &format!("{}, ", show)
    }

    let prompt = format!(
        "
    [LOOKING_FOR]
    {original_request}
    [LOOKING_FOR]

    [INPUT]
    {shows_as_string}
    [INPUT]

    "
    );
    let response = agent.prompt(prompt).await.unwrap();
    // let response = "(a, n, 0.5), (b, n, 0.5), (c, n, 0.5), (d, n, 0.5)";

    let Some(responses) = response
        .replace("(", "")
        .replace("),", ")")
        .split(")")
        .take(N)
        .map(|n| n.to_string())
        .collect_array::<N>()
    else {
        return Err("more then {ENTRIES} responses, exiting early.".into());
    };

    let mut show_like_perdictions = Vec::new();
    for (i, response) in responses.iter().enumerate() {
        let split_string = response
            .replace(" ", "")
            .split(",")
            .map(|n| n.to_string())
            .collect::<Vec<_>>();
        println!("split string is: {:#?}", split_string);
        let Some((show, would_like, confidence)) = split_string.iter().collect_tuple() else {
            println!(
                "show, would_like, and confidence, not found for {}?",
                response
            );
            continue;
        };

        // model returning malformed input for this field. so just setting it to this value instead.
        let show = shows
            .get(i)
            .map(|n| n.to_owned())
            .unwrap_or(show.to_owned().to_string());

        let perdiction = would_like.to_string();
        let confidence = confidence.to_string() + "e-01";

        let confidence: f32 = fast_float::parse(confidence.as_str()).unwrap_or(0.0);

        // for some reason, you need exponential notation to parse float to string.. this 10x the value to coax whatever float is given back into its original value
        let confidence = confidence * 10.0;

        show_like_perdictions.push(ShowOpinion {
            show: show.into(),
            perdiction,
            confidence,
        });
    }

    let Some(show_like_perdictions) = show_like_perdictions.as_array::<N>() else {
        return Err(format!(
            "show_like_perdictions is {} not {} long, exiting early",
            show_like_perdictions.len(),
            N
        ));
    };

    return Ok(show_like_perdictions.clone());
}

/// generate perdictions for interests in show with roberta model
pub async fn perdict_show_interests_roberta<const N: usize>(
    original_request: &String,
    shows: &[String; N],
    roberta: &RobertaModel,
) -> Result<[ShowOpinion; N], String> {
    let mut show_like_perdictions = Vec::new();

    for show in shows {
        let prompt = format!(
            "The user wants: {}. Would the user like {}? <mask>.",
            original_request, show
        );
        let roberta = roberta.clone();
        let Some(result) = (roberta.prompt::<1>(prompt).await.get(0)).map(|n| n.clone()) else {
            println!("could not get roberta result. continuing");
            continue;
        };

        let perdiction = ShowOpinion {
            show: show.clone(),
            perdiction: result.value,
            confidence: result.confidence,
        };
        show_like_perdictions.push(perdiction);
    }

    let Some(results) = show_like_perdictions.as_array::<N>() else {
        return Err(format!("could not get {ENTRIES} results"));
    };
    Ok(results.clone())
}

pub async fn generate_perdictive_graph(
    show_recommendation: &ShowRecommendation,
    agents: &AgentsCollection,
    user_interests: &Option<[ShowOpinion; ENTRIES]>,
    similar_shows: [String; ENTRIES],
) -> Result<(String, DataFrame, Option<Accuracy>), String> {
    let deepseek_perdictions = perdict_show_interests_deepseek(
        &show_recommendation.show,
        &similar_shows,
        &agents.show_appreciation_perdiction_agent,
    )
    .await?;

    let roberta_perdictions = perdict_show_interests_roberta(
        &show_recommendation.show,
        &similar_shows,
        &agents.roberta_agent,
    )
    .await?;

    let df = generate_perdictive_show_appreciation_scores_graph(
        &show_recommendation.show,
        &deepseek_perdictions,
        &roberta_perdictions,
        &user_interests,
    );
    println!("finished generating predictive graph");
    Ok(df)
}

pub fn bool_to_word(bool: bool) -> String {
    match bool {
        true => "yes".into(),
        false => "no".into(),
    }
}

/// graph that perdicts user interest in show
#[component]
pub fn PerdictiveGraph(
    recommended_show: Signal<ShowRecommendation>,
    agents: Signal<AgentsCollection>,
    similar_shows: Resource<Result<[String; ENTRIES], String>>,
) -> Element {
    let mut user_show_interests = use_context_provider(|| UserShowOpinions::default());

    let mut like_one = use_signal(|| None);
    let mut like_two = use_signal(|| None);
    let mut like_three = use_signal(|| None);
    let mut like_four = use_signal(|| None);

    let perdictive_graph = use_resource(move || async move {
        let Some(similar_shows) = similar_shows.read().cloned() else {
            return Err("similar shows not ready at tiem of generation".to_string());
        };
        let Ok(similar_shows) = similar_shows.inspect_err(|err| println!("{err}")) else {
            return Err(
                "similar shows could not generate at time of predictive graph generation".into(),
            );
        };

        // this prevents a double mut borrow from being occuring if the user sets this variable before the graph is finished loading.
        let user_show_interests = user_show_interests.0.read().clone().ok();

        Ok(generate_perdictive_graph(
            &*recommended_show.read(),
            &*agents.read(),
            &user_show_interests,
            similar_shows,
        )
        .await
        .unwrap())
    });

    let perdictive_graph_url = use_memo(move || {
        let fallback = None;
        let r = perdictive_graph.read();
        let Some(result) = r.as_ref() else {
            return fallback;
        };

        let Ok(result) = result else { return fallback };
        let svg = &result.0;

        return Some(load_chart_as_url(svg));
    });

    let shows = use_memo(move || {
        let fallback = ["???".to_string(), "???".into(), "???".into(), "???".into()];
        let similar_shows = similar_shows.read();
        let Some(similar_shows) = similar_shows.as_ref() else {
            return fallback;
        };
        let Ok(similar_shows) = similar_shows
            .clone()
            .inspect_err(|err| println!("COULDNT GET SIMILAR SHOWS: {err}"))
        else {
            return fallback;
        };

        similar_shows
    });

    let confirmed = use_memo(move || {
        let Some(one) = like_one.read().cloned() else {
            return None;
        };
        let Some(two) = like_two.read().cloned() else {
            return None;
        };
        let Some(three) = like_three.read().cloned() else {
            return None;
        };
        let Some(four) = like_four.read().cloned() else {
            return None;
        };

        Some([one, two, three, four])
    });

    let set_user_show_opinions = use_coroutine(move |mut rx: UnboundedReceiver<()>| async move {
        while let Some(_) = rx.next().await {
            let shows = &*shows.read();

            let Some(opinions) = confirmed.read().cloned() else {
                continue;
            };

            let Ok(mut use_show_interests) = user_show_interests
                .0
                .try_write()
                .inspect_err(|err| println!(" {}", err))
            else {
                continue;
            };

            *use_show_interests = Ok([
                ShowOpinion {
                    show: shows[0].clone(),
                    perdiction: bool_to_word(opinions[0]),
                    confidence: opinions[0] as u32 as f32,
                },
                ShowOpinion {
                    show: shows[1].clone(),
                    perdiction: bool_to_word(opinions[1]),
                    confidence: opinions[1] as u32 as f32,
                },
                ShowOpinion {
                    show: shows[2].clone(),
                    perdiction: bool_to_word(opinions[2]),
                    confidence: opinions[2] as u32 as f32,
                },
                ShowOpinion {
                    show: shows[3].clone(),
                    perdiction: bool_to_word(opinions[3]),
                    confidence: opinions[3] as u32 as f32,
                },
            ])
        }
    });

    let accuracy = use_memo(move || {
        let fallback = ("???".into(), "???".into());
        let r = perdictive_graph.read();
        let Some(df) = r.as_ref() else {
            return fallback;
        };
        let Ok((_svg, _df, accuracy)) = df
            .as_ref()
            .inspect_err(|err| println!("tried to update accuracy but: {err}"))
        else {
            return fallback;
        };
        let Some(accuracy) = accuracy else {
            return fallback;
        };

        return (accuracy.deepseek.to_string(), accuracy.roberta.to_string());
    });

    rsx! {
        h1 {
            u {
                "Predictive Method Graph: "
            }
        }

        h2 {
            u {
                "Brier Accuracy"
            }
            {
                ": "
            }
            {
                "(0-1: lower is better)"
            }
        }

        table {
            border: "2px solid grey",
            border_collapse: "collapse",
            margin_bottom: "16px",
            tr {
                th { "Model" }
                th { "Accuracy" }
            }
            tr {
                td { strong { "Deepseek V3.2" } }
                td { { format!("{}", accuracy.read().0) } }
            }
            tr {
                td { strong { "Roberta" } }
                td { { format!("{}", accuracy.read().1) } }
            }
        }
        div {
            {
                {
                    let url = &*perdictive_graph_url.read();
                    let element = match url {
                        Some(url) => rsx!(img {
                            src: url.as_str(),
                        }),
                        None => rsx!(
                            div {
                                display: "flex",
                                width: "500px",
                                height: "400px",
                                align_items: "center",
                                justify_content: "center",
                                background_color: "white",
                                h2 {
                                    text_align: "center",
                                    color: "black",
                                    "Loading graph..."
                                }
                            }
                        ),
                    };
                    element
                }
            }
        }

        div {
            hidden: user_show_interests.0.read().is_ok() == false || accuracy.read().0 != "???" ,

            h3 {
                "Show interests confirmed. Please await graph"
            }
        }

        div {
            hidden: user_show_interests.0.read().is_ok(),
            display: "flex",
            flex_direction: "column",
            h3 {
                hidden: user_show_interests.0.read().is_ok(),
                u {
                    "Show Interest Form"
                }
            }
            h3 {
                hidden: user_show_interests.0.read().is_ok(),

                "Select from the below or select all yes or no"
            }
            div {
                hidden: user_show_interests.0.read().is_ok(),
                div {
                    button {
                        onmouseup: move |_event| {
                            *like_one.write() = Some(true);
                            *like_two.write() = Some(true);
                            *like_three.write() = Some(true);
                            *like_four.write() = Some(true);

                        },
                        "all yes"
                    }
                    button {
                        onmouseup: move |_event| {
                            *like_one.write() = Some(false);
                            *like_two.write() = Some(false);
                            *like_three.write() = Some(false);
                            *like_four.write() = Some(false);

                        },
                        "all no"
                    }
                    button {
                        disabled: confirmed.read().is_none(),
                        onmouseup: move |_event | {
                            set_user_show_opinions.send(());
                        },
                        "confirm"
                    }
                }
                div {
                    h3 {
                        {format!("do you like: {}", shows.read()[0])}
                    }
                    button {
                        class: active_or(like_one.read().unwrap_or(false), Styles::selected, ""),
                        onmouseup: move |_event| {
                            *like_one.write() = Some(true)
                        },
                        "yes"
                    }
                    button {
                        class: active_or(!like_one.read().unwrap_or(true), Styles::selected, ""),

                        onmouseup: move |_event| {
                            *like_one.write() = Some(false)
                        },
                        "no"
                    }
                }
                div {
                    h3 {
                        {format!("do you like: {}", shows.read()[1])}
                    }
                    button {
                        class: active_or(like_two.read().unwrap_or(false), Styles::selected, ""),

                        onmouseup: move |_event| {
                            *like_two.write() = Some(true)
                        },
                        "yes"
                    }
                    button {
                        class: active_or(!like_two.read().unwrap_or(true), Styles::selected, ""),

                        onmouseup: move |_event| {
                            *like_two.write() = Some(false)
                        },
                        "no"
                    }
                }
                div {
                    h3 {
                        {format!("do you like: {}", shows.read()[2])}
                    }
                    button {
                        class: active_or(like_three.read().unwrap_or(false), Styles::selected, ""),

                        onmouseup: move |_event| {
                            *like_three.write() = Some(true)
                        },
                        "yes"
                    }
                    button {
                        class: active_or(!like_three.read().unwrap_or(true), Styles::selected, ""),

                        onmouseup: move |_event| {
                            *like_three.write() = Some(false)
                        },
                        "no"
                    }
                }
                div {
                    h3 {
                        {format!("do you like: {}", shows.read()[3])}
                    }
                    button {
                        class: active_or(like_four.read().unwrap_or(false), Styles::selected, ""),

                        onmouseup: move |_event| {
                            *like_four.write() = Some(true)
                        },
                        "yes"
                    }
                    button {
                        class: active_or(!like_four.read().unwrap_or(true), Styles::selected, ""),

                        onmouseup: move |_event| {
                            *like_four.write() = Some(false)
                        },
                        "no"
                    }
                }
            }

        }
    }
}
