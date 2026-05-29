//! Optional ONNX Runtime embeddings when `embedding-local` feature is enabled.

use super::error::EmbeddingError;

/// Attempts ONNX inference; returns error when feature disabled or model unavailable.
pub async fn embed_batch(
    model_path: &str,
    texts: &[String],
    dimensions: usize,
) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    #[cfg(feature = "embedding-local")]
    {
        return try_ort_embed(model_path, texts, dimensions).await;
    }
    let _ = (model_path, texts, dimensions);
    Err(EmbeddingError::RequestFailed {
        reason: "build with --features embedding-local for ONNX embeddings".into(),
    })
}

#[cfg(feature = "embedding-local")]
async fn try_ort_embed(
    model_path: &str,
    texts: &[String],
    dimensions: usize,
) -> Result<Vec<Vec<f32>>, EmbeddingError> {
    if !std::path::Path::new(model_path).exists() {
        return Err(EmbeddingError::RequestFailed {
            reason: format!("ONNX model not found at '{model_path}'"),
        });
    }
    use ort::session::Session;
    use ort::value::Tensor;

    let session = Session::builder()
        .map_err(|e| onnx_err(format!("session builder: {e}")))?
        .commit_from_file(model_path)
        .map_err(|e| onnx_err(format!("load model: {e}")))?;

    let mut out = Vec::with_capacity(texts.len());
    for text in texts {
        let tokens: Vec<i64> = text
            .split_whitespace()
            .enumerate()
            .map(|(idx, _)| idx as i64 + 1)
            .collect();
        if tokens.is_empty() {
            out.push(super::local::local_embed("", dimensions));
            continue;
        }
        let input = Tensor::from_array(([1_usize, tokens.len()], tokens))
            .map_err(|e| onnx_err(format!("tensor: {e}")))?;
        let outputs = session
            .run(ort::inputs![input])
            .map_err(|e| onnx_err(format!("run: {e}")))?;
        let (_name, value) = outputs.iter().next().ok_or_else(|| {
            onnx_err("model returned no outputs".into())
        })?;
        let (_shape, slice) = value
            .try_extract_tensor::<f32>()
            .map_err(|e| onnx_err(format!("extract: {e}")))?;
        let mut vec = vec![0.0_f32; dimensions];
        let take = dimensions.min(slice.len());
        vec[..take].copy_from_slice(&slice[..take]);
        super::local::l2_normalize_slice(&mut vec);
        out.push(vec);
    }
    Ok(out)
}

#[cfg(feature = "embedding-local")]
fn onnx_err(reason: String) -> EmbeddingError {
    EmbeddingError::RequestFailed { reason }
}

#[cfg(all(test, feature = "embedding-local"))]
mod onnx_tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires ARCFLOW_EMBEDDING_ONNX_PATH"]
    async fn onnx_embed_smoke() {
        let path = std::env::var("ARCFLOW_EMBEDDING_ONNX_PATH").expect("model path");
        let v = embed_batch(&path, &["hello".into()], 384)
            .await
            .expect("onnx embed");
        assert_eq!(v[0].len(), 384);
    }
}
