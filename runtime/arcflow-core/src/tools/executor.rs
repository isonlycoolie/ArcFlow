//! Tool invocation with async timeout.

use std::pin::Pin;

use serde_json::Value;

use super::error::ToolError;

/// Invokes a tool implementation (Python binding or test double).
pub trait ToolInvoker: Send + Sync {
    /// Runs the tool synchronously; called from a blocking task inside async timeout.
    fn invoke(&self, name: &str, input: &Value) -> Result<String, ToolError>;
}

type BoxedFut = Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send>>;

/// Blocking-friendly wrapper for [`ToolInvoker`].
pub fn spawn_invoke(
    invoker: std::sync::Arc<dyn ToolInvoker>,
    name: String,
    input: Value,
) -> BoxedFut {
    Box::pin(async move {
        let name_for_err = name.clone();
        let task = tokio::task::spawn_blocking(move || invoker.invoke(&name, &input));
        match task.await {
            Ok(Ok(out)) => Ok(out),
            Ok(Err(e)) => Err(e),
            Err(join_err) => Err(ToolError::ExecutionFailed {
                name: name_for_err,
                step_id: None,
                reason: join_err.to_string(),
            }),
        }
    })
}
