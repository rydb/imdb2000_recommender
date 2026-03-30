use crate::model::{BertMaskedLM, BertMaskedLMRecord, BertModelConfig};
use burn::backend::wgpu::WgpuDevice;
use burn::backend::Wgpu;

use super::data::{BertInputBatcher, BertTokenizer};
use super::fill_mask::fill_mask;
use super::loader::{download_hf_model, load_model_config};
use burn::data::dataloader::batcher::Batcher;
use burn::module::Module;
use std::sync::Arc;



#[derive(Clone)]
pub struct RobertaModel {
    tokenizer: Arc<BertTokenizer>,
    batcher: Arc<BertInputBatcher>,
    model: BertMaskedLM<Wgpu>,
    model_config: BertModelConfig,
    device: WgpuDevice,
}

#[derive(Clone, Debug)]
pub struct RobertaResult {
    pub value: String,
    pub confidence: f32,
}

impl RobertaModel {
    pub async fn new() -> Self {
        type B = Wgpu;
        let device: WgpuDevice = WgpuDevice::default();
        let default_model = "roberta-base".to_string();

        let model_variant = &default_model;

        let (config_file, model_file) = download_hf_model(model_variant).await;
        let model_config = load_model_config(config_file);

        let model_record: BertMaskedLMRecord<B> =
            BertMaskedLM::from_safetensors(model_file, &device, model_config.clone());

        let model = model_config
            .init_with_lm_head(&device)
            .load_record(model_record);

        let tokenizer = Arc::new(BertTokenizer::new(
            model_variant.to_string(),
            model_config.pad_token_id,
        ));

        // Batch the input samples to max sequence length with padding
        let batcher = Arc::new(BertInputBatcher::new(
            tokenizer.clone(),
            model_config.max_seq_len.unwrap(),
        ));
        Self {
            tokenizer,
            batcher,
            model,
            device,
            model_config,
        }
    }
    pub async fn prompt<const ENTRIES: usize>(self, input: String) -> [RobertaResult; ENTRIES] {
        // putting prompt inside of a seperate async thread to stop a long compute time blocking the main app.
        let result = tokio::spawn(async move {
            let text_samples = [input];
            let input = self.batcher.batch(
                text_samples.clone().map(|n| n.to_owned()).to_vec(),
                &self.device,
            );
            let [batch_size, _seq_len] = input.tokens.dims();
            println!("Input: {:?} // (Batch Size, Seq_len)", input.tokens.shape());

            let output = fill_mask(
                &self.model,
                &self.model_config,
                self.tokenizer.as_ref(),
                input,
            );

            let mut results = Vec::new();
            for i in 0..batch_size {
                let input = &text_samples[i];
                let result = &output[i];
                println!("Input: {}", input);
                for fill_mask_result in result.iter() {
                    let top_k = &fill_mask_result.top_k;
                    for (_k, (score, token)) in top_k.iter().enumerate() {
                        results.push(RobertaResult {
                            value: token.clone(),
                            confidence: *score,
                        });
                    }
                }
            }
            let result_taken = results
                .iter()
                .take(ENTRIES)
                .map(|n| n.clone())
                .collect::<Vec<_>>();
            let arr = result_taken
                .as_array()
                .map(|n| n.to_owned())
                .unwrap();

            // let arr = results.as_array::<ENTRIES>().map(|n| n.to_owned()).unwrap();
            arr
        })
        .await
        .unwrap();
        result
    }
}
