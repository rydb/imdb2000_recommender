use super::*;

#[component]
pub fn DescriptiveGraph(
    recommended_show: Signal<ShowRecommendation>,
    agents: Signal<AgentsCollection>,
) -> Element {
    let descriptive_graph_url = use_resource(move || async move {
        let svg = generate_descriptive_theme_confidence_graph::<ENTRIES>(
            agents.read().roberta_agent.clone(),
            &*recommended_show.read().show,
        )
        .await;
        load_chart_as_url(&svg)
    });

    rsx! {
        h1 {
            u {
                "Descriptive Method Graph: "
            }
        }
        img {
            src: descriptive_graph_url,
        }
    }
}
