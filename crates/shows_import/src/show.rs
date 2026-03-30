use burn::tensor::{Bool, Int, Tensor, backend::Backend};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowRecord {
    pub id: String,
    pub title: String,
    pub original_title: String,
    pub overview: String,
    pub premiere_date: Option<String>,
    pub genre: String,
    pub country_origin: Option<String>,
    pub original_language: String,
    pub rating: Option<f64>,
    pub votes: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SimilarityBatch<B: Backend> {
    pub text1_input_ids: Tensor<B, 2, Int>,
    pub text1_attention_mask: Tensor<B, 2, Bool>,
    pub text2_input_ids: Tensor<B, 2, Int>,
    pub text2_attention_mask: Tensor<B, 2, Bool>,
    pub labels: Tensor<B, 1, Int>, // For semantic similarity labels
}

#[derive(Debug)]
pub struct ShowSimilarityDataset {
    pub shows: Vec<ShowRecord>,
    pub pairs: Vec<SimilarityPair>,
}

#[derive(Debug, Clone)]
pub struct SimilarityPair {
    pub text1: String,
    pub text2: String,
    pub label: i32, // 0 = dissimilar, 1 = similar (you might want to adjust this)
}
