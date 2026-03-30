use dioxus::prelude::*;
use rig::{
    agent::Agent,
    client::{BearerAuth, CompletionClient, ProviderClient},
    providers::{
        self,
        deepseek::{self},
    },
};
use roberta::show_prediction::RobertaModel;

use crate::ui::Signal;

/// should be long enough for most responses
pub const MAX_WORDS: u64 = 20;

/// most words = 1-2 tokens, this should be long enough to ensure an adequete response.
pub const MAX_TOKENS: u64 = MAX_WORDS * 5;

pub const TRUE_STRING: &'static str = "TRUE";
pub const FALSE_STRING: &'static str = "FALSE";

// pub struct RobertaAgent {
//     /// model for roberta BERT model. Tuned for show recommendations.
//     pub roberta_agent: RobertaModel,
// }

#[derive(Clone)]
pub struct AgentsCollection {
    _client: deepseek::Client,
    /// cleans prompts to stop user prompt injection/to confirm its on topic
    pub cleaner_agent: Agent<deepseek::CompletionModel>,
    /// recommends a single show. Formatted for user eyes.
    pub single_show_recommender_agent: Agent<deepseek::CompletionModel>,
    /// generates 4 shows to be recommended based on a given showw, formatted for parsing.
    pub four_similiar_show_recommender_agent: Agent<deepseek::CompletionModel>,
    /// perdicts weather user would like given recommended shows.
    pub show_appreciation_perdiction_agent: Agent<deepseek::CompletionModel>,

    pub roberta_agent: RobertaModel,
}

impl AgentsCollection {
    pub fn new(key: &str, roberta: RobertaModel) -> Self {
        let client = providers::deepseek::Client::from_val(BearerAuth::from(key));
        // agent that cleans a prompt to ensure that is actually a request for a movie/show and not something un-releated/prompt injection
        let cleaner_agent = client
            .agent(deepseek::DEEPSEEK_CHAT)
            .preamble(&format!(
                "Your job is to take prompt and confirm that it is a request for a show/movie 
        recommendation. If it is, respond {TRUE_STRING}. If its not, respond {FALSE_STRING}
        
        Do not enumerate. Respond ONLY with either {TRUE_STRING} or {FALSE_STRING}.
        "
            ))
            .max_tokens(MAX_TOKENS)
            .build();

        // agent that takes confirmed clean response and then recommends a show based on it.
        let single_show_recommender_agent = client.agent(deepseek::DEEPSEEK_CHAT)
        .preamble(
            "
            Your job is to respond with a recommendion for a singular show or movie from the IMDB top 2000
            based on the given prompt with the given format. Encase each section inside of curly braces:

            [SHOW]

            [SYNOPSIS]

            [REASON]
            "
        )
        .max_tokens(MAX_TOKENS)
        .build();

        // [CALL_TO_ACTION]
        // Want another suggestion?
        // [CALL_TO_ACTION]

        let four_similiar_show_recommender_agent = client
            .agent(deepseek::DEEPSEEK_CHAT)
            .preamble(
                "
            recommended four shows from the imdb top 2000 similar to the given show
            inside of parenthesis formatted as the below provided section [START] to [END]. 
            Do not elaborate or add anything else.

            [SHOW]
            show
            [SHOW]

            [START]
            a, b, c, d
            [END]
            ",
            )
            .max_tokens(MAX_TOKENS)
            .build();

        let show_appreciation_perdiction_agent = client.agent(deepseek::DEEPSEEK_CHAT)
        .preamble(
            "
            given the following [INPUT] shows and user [LIKES], give a yes or no response in addition to a [1.0 to 0.0] confidence rating on your answer on weather or
            not the user would like the given shows according to the below [OUTPUT]. Do not elaborate or add anything else.

            [LOOKING_FOR]
            description
            [LOOKING_FOR]

            [INPUT]
            a, b, c, d
            [INPUT]

            [OUTPUT]
            (a, y/n, rating), (b, y/n, rating), (c, y/n, rating), (a, y/n, rating)
            [OUTPUT]
            "
        )
        .max_tokens(MAX_TOKENS * 3)
        .build()
        ;
        println!("agents initialized");
        Self {
            _client: client,
            cleaner_agent,
            single_show_recommender_agent,
            four_similiar_show_recommender_agent,
            show_appreciation_perdiction_agent,
            roberta_agent: roberta,
        }
    }
}

#[derive(Clone)]
pub struct Agents(pub Signal<Result<Signal<AgentsCollection>, String>>);

impl Default for Agents {
    fn default() -> Self {
        Self(Signal::new(Err("Agents uninitialized".into())))
    }
}
