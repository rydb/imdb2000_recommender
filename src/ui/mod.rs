#![forbid(unstable_features)]

use dioxus::{
    html::{FileData, HasFileData},
    prelude::*,
};
use dioxus_desktop::{LogicalSize, use_window};
use futures_util::StreamExt;
use itertools::Itertools;
use rig::completion::Prompt;
use roberta::show_prediction::RobertaModel;
use std::{fmt::Display, rc::Rc};

pub mod debug_menu;
pub use debug_menu::*;

pub mod graphs;
pub use graphs::*;

pub mod agents;
pub use agents::*;

pub mod drag_elements;
pub use drag_elements::*;

pub mod horizontal_scroll;
pub use horizontal_scroll::*;

// pub type Signal<T> = Signal<T, UnsyncStorage>;

#[derive(Clone, Debug, PartialEq)]
enum KeyState {
    Recieved(String),
    Waiting(String),
    Invalid(String),
}

#[derive(Clone, Debug, Default)]
pub enum PromptStatus {
    Success(String),
    Error(String),
    Thinking,
    #[default]
    WaitingForInput,
}

#[derive(Clone, Default)]
pub struct PromptOutput(Signal<PromptStatus>);

impl Display for PromptOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.read())
    }
}

impl Display for PromptStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            PromptStatus::Success(n) => n,
            PromptStatus::Error(n) => n,
            PromptStatus::Thinking => "thinking...",
            PromptStatus::WaitingForInput => "...",
        };
        write!(f, "{display}")
    }
}

#[derive(Clone)]
pub struct ShowRecommendation {
    show: String,
    _synopsis: String,
    _reason: String,
}

#[derive(Clone)]
pub struct SingleShowRecommendation(Signal<Result<Signal<ShowRecommendation>, String>>);

impl Default for SingleShowRecommendation {
    fn default() -> Self {
        Self(Signal::new(Err("recommendation not generated".into())))
    }
}

#[derive(Default, Clone)]
pub struct ShowRequestPrompt(Signal<String>);

pub fn err_rsx(err: &str) -> Result<VNode, RenderError> {
    let element = rsx!(h3 {
        {
            format!("Could not render element: {err}")
        }
    });
    element
}

#[derive(Clone)]
pub struct AppDocument(pub Signal<Rc<dyn dioxus_document::Document>>);

// TODO: doesn't work without dx bundle. re-add if issue gets resolved.
// #[css_module("assets/themes/dark_mode.css")]
// pub struct Styles;
pub struct Styles;

// TODO: replace this with #[embedded_css_module] on [`Styles`] or similar if it gets added.
// see: https://github.com/DioxusLabs/dioxus/issues/5364
#[allow(non_upper_case_globals)] // for #[css_module] equivalent syntax
impl Styles {
    const selected: &str = "selected";
    const card: &str = "card";
}

/// returns the given cs style if the given bool is true, or uses the second given style if its not.
pub fn active_or(bool: bool, active: impl ToString, inactive: impl ToString) -> String {
    if bool {
        active.to_string()
    } else {
        inactive.to_string()
    }
}

#[component]
pub fn app() -> Element {
    let mut show_request = use_context_provider(|| ShowRequestPrompt::default());

    let mut show_recommendation = use_context_provider(|| SingleShowRecommendation::default());

    let mut agents = use_context_provider(|| Agents::default());

    let mut prompt_output = use_context_provider(|| PromptOutput::default());

    let mut key_state = use_signal(|| KeyState::Waiting("...".to_string()));

    let mut words_left = use_signal(|| MAX_WORDS as isize);

    let mut debug_menu_active = use_signal(|| false);

    let mut file_input_hidden = use_signal(|| false);

    let _charts = use_context_provider(|| Signal::new(0));

    let mut roberta_model_signal: Signal<Option<RobertaModel>> = use_signal(|| None);

    let mut roberta_model_loaded = use_signal(|| false);

    let sample_prompt = use_signal(|| "something with drama");

    let _document = use_context_provider(|| AppDocument(Signal::new(dioxus_document::document())));

    use_window().set_min_inner_size(Some(LogicalSize::new(1200, 1350)));

    let initialize_agents = use_coroutine(move |mut rx: UnboundedReceiver<()>| async move {
        while let Some(_) = rx.next().await {
            let key_state = key_state.read().clone();
            let KeyState::Recieved(key) = key_state else {
                println!("Cannot initialize models, keysate is: {:#?}", key_state);
                continue;
            };
            let Some(roberta) = roberta_model_signal.read().cloned() else {
                println!("roberta not loaded yet, exiting agent initialization");
                continue;
            };
            let collection = AgentsCollection::new(&key, roberta);

            println!("finished initializing agents");
            *agents.0.write() = Ok(Signal::new(collection));
        }
    });
    use_future(move || async move {
        let handle = tokio::spawn(async move {
            let roberta_model = RobertaModel::new().await;
            roberta_model
        });
        let result = handle.await.unwrap();

        *roberta_model_signal.write() = Some(result);
        *roberta_model_loaded.write() = true;
        if matches!(*key_state.read(), KeyState::Recieved(_)) {
            initialize_agents.send(());
        }
    });

    let key_setter = use_coroutine(
        move |mut rx: UnboundedReceiver<Result<String, String>>| async move {
            while let Some(key_as_string) = rx.next().await {
                match key_as_string {
                    Ok(key) => {
                        *key_state.write() = KeyState::Recieved(key);
                        *file_input_hidden.write() = true;
                        if *roberta_model_loaded.read() == true {
                            initialize_agents.send(());
                        }
                    }
                    Err(err) => {
                        *key_state.write() = KeyState::Invalid(err.to_string());
                    }
                }
            }
        },
    );

    let key_file_through_path =
        use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
            while let Some(path) = rx.next().await {
                let result = std::fs::read_to_string(&path).map_err(|err| {
                    "Could not pre-load model from path. Drag an drop key to load model: "
                        .to_string()
                        + &err.to_string()
                        + " path: "
                        + &path
                });
                key_setter.send(result)
            }
        });
    let key_file_sender = use_coroutine(move |mut rx: UnboundedReceiver<FileData>| async move {
        // try initially setting key through file system for convienience.
        println!("setting key through path");
        key_file_through_path.send("key/capstone_key.txt".into());
        while let Some(file) = rx.next().await {
            key_setter.send(file.read_string().await.map_err(|err| err.to_string()));
        }
    });

    let prompt_deepseek_for_show = use_coroutine(
        move |mut rx: UnboundedReceiver<String>| async move {
            while let Some(prompt) = rx.next().await {
                let Ok(agents_ref) = agents
                    .0
                    .read()
                    .clone()
                    .inspect_err(|err| println!("could not prompt for show: {err}"))
                else {
                    continue;
                };
                let agents = agents_ref.read();

                *prompt_output.0.write() = PromptStatus::Thinking;
                let check = agents.cleaner_agent.prompt(&prompt).await;

                match check {
                    Ok(cleaner_response) => {
                        if cleaner_response == { TRUE_STRING } {
                        } else if cleaner_response == { FALSE_STRING } {
                            *prompt_output.0.write() = PromptStatus::Error(
                                "Invalid prompt.. Please enter a movie/show recommendation"
                                    .to_string(),
                            );
                            continue;
                        } else {
                            *prompt_output.0.write() = PromptStatus::Error("prompt malformed or is leading to a malformed response. Try again with a different prompt".to_string());
                        }
                    }
                    Err(err) => {
                        *prompt_output.0.write() = PromptStatus::Error(format!("{err}").into())
                    }
                }

                let recommendation = agents.single_show_recommender_agent.prompt(&prompt).await;
                let result = match recommendation {
                    Ok(n) => n,
                    Err(err) => err.to_string(),
                };

                let Some((show, synopsis, reason)) = result
                    .splitn(3, "}")
                    .map(|n| {
                        let n = n.replace("{", "").replace("}", "").to_owned();
                        n
                    })
                    .collect_tuple()
                else {
                    println!("result did not split as expected. result: {result}");
                    continue;
                };

                *show_recommendation.0.write() = Ok(Signal::new(ShowRecommendation {
                    show: show.clone(),
                    _synopsis: synopsis.clone(),
                    _reason: reason.clone(),
                }));

                *prompt_output.0.write() = PromptStatus::Success(format!(
                    "
                Show: {show}\n, 
                Reason: {reason}\n
                synopsis: {synopsis}\n

                Want another recommendation?
                "
                ));
            }
        },
    );

    let deepseek_status_string = use_memo(move || {
        let result = match &*key_state.read() {
            KeyState::Recieved(_key) => ("☺️ 🟩: loaded".to_string(), "".to_string()),
            KeyState::Waiting(err) => ("😐 🟧: waiting".into(), format!("Waiting for key.. Drag and drop the key from the file explorer to \"drag and drop key here\" Actual error: {err}").into()),
            KeyState::Invalid(err) => ("😢 🟥: error".into(), format!("Could not load model. Actual error {err}")),
        };
        result
    });

    let roberta_status_string = use_memo(move || {
        let result = match &*roberta_model_loaded.read() {
            true => ("☺️ 🟩: loaded".to_string(), "".to_string()),
            false => (
                "⌛ 🟧: loading...".into(),
                "This may take longer on slower machines.".into(),
            ),
        };
        result
    });

    let main_app_element = rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/themes/dark_mode.css")  }

        div {
            button {
                class: active_or(*debug_menu_active.read(), Styles::selected, ""),
                onclick: move |_|  {
                    *debug_menu_active.write() ^= true;
                },
                "Debug Menu",
            }
            div {
                user_select: "none",
                DebugMenu { active: debug_menu_active }
            }
        }

        body {
            display: "flex",
            align_items: "flex-start",
            justify_content: "flex-start",
            flex_wrap: "nowrap",
            flex_grow: 0,
            gap: "40px",
            z_index: -1,
            div {
                flex_shrink: 0,
                overflow: "hidden",
                h1 {
                    text_align: "center",
                    u {
                        "IMDB top 2000 Recommender!"

                    }
                }
                button {
                    hidden: *file_input_hidden.read(),
                    ondrop: move |event| {
                        println!("CURRENT FILES: {:#?} ",event.files());
                        if let Some(file) = event.files().last() {
                            key_file_sender.send(file.clone());
                        };
                    },
                    "drag and drop key here",
                }
                div {
                    display: "flex",
                    flex_direction: "column",
                    align_items: "center",

                    table {
                        width: "500px",

                        tr {
                            th { "Models" }
                            th { "Status" }
                            th { "Additional Notes" }
                        }
                        tr {
                            td { "Deepseek" }
                            td {
                                {
                                    deepseek_status_string.read().0.clone()

                                }
                            }
                            td {
                                {
                                    deepseek_status_string.read().1.clone()
                                }
                            }
                        }
                        tr {
                            td { "Roberta" }
                            td {
                                {
                                    roberta_status_string.read().0.clone()
                                }
                            }
                            td {
                                {
                                    roberta_status_string.read().1.clone()
                                }
                            }
                        }
                    }
                }

                p {
                    text_align: "center",
                    "What kind of show are you looking for?(20 words max)"
                }
                p {
                    text_align: "center",
                    b {
                        "(click the input box and press enter to submit prompt)"
                    }
                }
                div {
                    display: "flex",
                    flex: "0 0 0px",
                    align_items: "center",
                    flex_direction: "column",
                    justify_content: "center",
                    height: "auto",
                    gap: "20px",
                    button {
                        onmouseup: move |_event| {
                            *show_request.0.write() = (*sample_prompt.read()).to_string()
                        },
                        "Enter Sample Prompt"
                    }
                    div {
                        textarea {
                            height: "auto",
                            min_width: "100px",
                            min_height: "100px",
                            resize: "none",
                            value: show_request.0.read().cloned(),

                            oninput: move |event| {
                                *show_request.0.write() = event.value();
                                *words_left.write() = MAX_WORDS as isize - show_request.0.read().split_whitespace().count() as isize;
                            },

                            onkeyup: move |event| {
                                if event.code().to_string() == "Enter" {
                                    {
                                        let mut errors = Vec::new();
                                        if *words_left.read() <= 0 {
                                            errors.push("Prompt too long\n")
                                        }
                                        if !matches!(*key_state.read(), KeyState::Recieved(_))  {
                                            errors.push("Deepseek not loaded. ");
                                        }
                                        if *roberta_model_loaded.read() == false{
                                            errors.push("Roberta not loaded.\n");
                                        }

                                        if errors.is_empty() == false {
                                            let mut error_string = "".to_owned();
                                            for error in errors {
                                                error_string += error;
                                            }
                                            *prompt_output.0.write() = PromptStatus::Error("Cannot execute prompt yet: \n\n".to_owned() + &error_string + "\n Resolve the issue(s) listed in \"Additional Notes\" and try again.");
                                        }
                                        else {
                                            prompt_deepseek_for_show.send(show_request.0.read().to_string());
                                        }
                                    }

                                }
                            }
                        }

                    }
                    div {
                        display: "flex",
                        justify_content: "center",
                        align_items: "center",

                        h4 {
                            {"words left: ".to_owned() + &words_left.read().to_string()}
                        }
                    }
                                b {}
                div {
                    max_width: "300px",

                    text_align: "center",
                    h1 {
                        text_align: "center",
                        "Result"
                    }
                    p {
                        overflow_wrap: "break-word",
                        {format!("{prompt_output}")}
                    }
                }
                }
            }
            div {
                GraphOptions {  }

            }
        }
    };
    // add drag surface to app
    let post_layers_element = rsx! {
            DragSurfaceLayer { element: main_app_element }
    };
    post_layers_element
}
