use charton::prelude::LayeredChart;

pub mod bargraph;
pub mod descriptive;
pub mod donut_chart;
pub mod pie_chart;
pub mod predictive;

pub const DATABASE_AS_STR: &str = include_str!("../../assets/datasets/top_rated_2000webseries.csv");

pub trait DarkTheme {
    fn apply_dark_theme(self) -> Self;
}

impl DarkTheme for LayeredChart {
    fn apply_dark_theme(self) -> Self {
        //TODO: implement this after charton legend color gets merged into main
        // self
        // .with_background("#2a2a2a")
        // .with_label_color("#ffffff")
        // .with_tick_label_color("#ffffff")
        // .with_title_color("#ffffff")
        self
    }
}
