use super::{TaskGraph, Image};

pub trait PromptSerialize {
    /// Convert the `TaskGraph` to a string that can be used as a prompt for a language model.
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String>;
}

/// Reply from the language model
pub trait PromptDeserialize {
    /// Extract image from the reply from the language model.
    fn image(&self) -> anyhow::Result<Image>;

    /// Extract status message from the reply from the language model.
    /// 
    /// What kind of problems were there interpreting the reply.
    fn status(&self) -> Option<String>;
}

#[derive(Debug, Clone)]
pub enum PromptType {
    ShapeAndTransformConnectivity4,
    ShapeAndTransformConnectivity8,
    RunLengthEncoding,
}
