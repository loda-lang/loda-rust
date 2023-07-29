use super::TaskGraph;

pub trait PromptSerialize {
    /// Convert the `TaskGraph` to a string that can be used as a prompt for a language model.
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String>;
}

#[derive(Debug, Clone)]
pub enum PromptType {
    ShapeAndTransformConnectivity4,
    ShapeAndTransformConnectivity8,
    RunLengthEncoding,
}
