use std::io::Cursor;
use std::sync::{Mutex, OnceLock};

use tokenizers::Tokenizer;
use tract_onnx::prelude::*;

use dp_sync::LockExt;

/// all-MiniLM-L6-v2, int8 quantized ONNX export (Apache-2.0; see `THIRD-PARTY-NOTICES.md`).
/// Bundled so patch notes search works fully offline with no per-query network or API cost.
const MODEL_BYTES: &[u8] = include_bytes!("../../../assets/patch-search/minilm-l6-v2-int8.onnx");
const TOKENIZER_BYTES: &[u8] = include_bytes!("../../../assets/patch-search/tokenizer.json");

type Model = TypedRunnableModel<TypedModel>;

/// all-MiniLM-L6-v2's fixed output width. Only used to build a zero vector for a line that must
/// never surface in semantic search (see `store::merge_lines`'s image-marker case) without paying
/// for a real model call — cosine similarity against an all-zero vector is always exactly 0, so it
/// can never cross `search::SEMANTIC_FLOOR`.
pub const EMBEDDING_DIM: usize = 384;

pub struct Embedder {
    model: Model,
    tokenizer: Tokenizer,
    /// SAFETY: the `tokenizers` crate's "onig" backend is not safe to call concurrently from
    /// multiple threads on one `Tokenizer` (observed as a crash, not just a slowdown, when
    /// `cargo test` ran several `embed()` calls in parallel). One embed at a time is plenty fast
    /// for interactive search; this trades unneeded parallelism for not crashing.
    call_lock: Mutex<()>,
}

impl Embedder {
    fn load() -> Result<Self, String> {
        let model = tract_onnx::onnx()
            .model_for_read(&mut Cursor::new(MODEL_BYTES))
            .and_then(|m| m.into_optimized())
            .and_then(|m| m.into_runnable())
            .map_err(|e| format!("could not load the patch search model: {e}"))?;
        let tokenizer = Tokenizer::from_bytes(TOKENIZER_BYTES)
            .map_err(|e| format!("could not load the patch search tokenizer: {e}"))?;
        Ok(Self { model, tokenizer, call_lock: Mutex::new(()) })
    }

    /// Mean-pooled, L2-normalised sentence embedding. Cosine similarity between two normalised
    /// vectors is then just their dot product (see `cosine`).
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let _guard = self.call_lock.lock_or_recover();
        let encoding = self.tokenizer.encode(text, true).map_err(|e| e.to_string())?;
        let ids: Vec<i64> = encoding.get_ids().iter().map(|&x| i64::from(x)).collect();
        let mask: Vec<i64> = encoding.get_attention_mask().iter().map(|&x| i64::from(x)).collect();
        let type_ids: Vec<i64> = encoding.get_type_ids().iter().map(|&x| i64::from(x)).collect();
        let n = ids.len();

        let input_ids = Tensor::from_shape(&[1, n], &ids).map_err(|e| e.to_string())?;
        let attention_mask = Tensor::from_shape(&[1, n], &mask).map_err(|e| e.to_string())?;
        let token_type_ids = Tensor::from_shape(&[1, n], &type_ids).map_err(|e| e.to_string())?;

        let outputs = self
            .model
            .run(tvec!(input_ids.into(), attention_mask.into(), token_type_ids.into()))
            .map_err(|e| e.to_string())?;
        let hidden = outputs[0].to_array_view::<f32>().map_err(|e| e.to_string())?;
        let dim = hidden.shape()[2];
        debug_assert_eq!(dim, EMBEDDING_DIM, "EMBEDDING_DIM must track the bundled model's real output width");

        let mut pooled = vec![0f32; dim];
        for (t, &m) in mask.iter().enumerate() {
            let w = m as f32;
            for d in 0..dim {
                pooled[d] += hidden[[0, t, d]] * w;
            }
        }
        let total: f32 = mask.iter().sum::<i64>() as f32;
        for v in &mut pooled {
            *v /= total.max(1.0);
        }
        let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        for v in &mut pooled {
            *v /= norm.max(1e-9);
        }
        Ok(pooled)
    }
}

/// Both inputs are expected L2-normalised, so their dot product is the cosine similarity.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Loaded once, lazily, the first time a search needs it: parsing and optimising the graph takes
/// real time and most commands never touch the embedding layer at all.
static EMBEDDER: OnceLock<Result<Embedder, String>> = OnceLock::new();

pub fn embedder() -> Result<&'static Embedder, &'static str> {
    match EMBEDDER.get_or_init(Embedder::load) {
        Ok(e) => Ok(e),
        Err(e) => {
            log::warn!("{e}");
            Err("The semantic search model could not be loaded.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_of_identical_normalised_vectors_is_one() {
        let v = [0.6, 0.8];
        assert!((cosine(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_of_orthogonal_vectors_is_zero() {
        assert_eq!(cosine(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
    }

    /// Also doubles as a smoke test that the ONNX graph and tokenizer are actually compatible.
    #[test]
    fn the_bundled_model_embeds_related_text_closer_than_unrelated_text() {
        let embedder = embedder().unwrap();
        let query = embedder.embed("Pocket's Affliction full healing removal").unwrap();
        let related = embedder.embed("Pocket: Affliction T3 now gives -100% healing reduction").unwrap();
        let unrelated = embedder.embed("Paige: Fixed some collision issues with Rallying Charge").unwrap();
        assert!(cosine(&query, &related) > cosine(&query, &unrelated));
        assert!(cosine(&query, &related) > 0.5, "expected a strong match, got {}", cosine(&query, &related));
    }
}
