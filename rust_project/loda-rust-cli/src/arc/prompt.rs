use super::TaskGraph;

pub trait PromptSerialize {
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String>;
}

#[derive(Debug, Clone)]
pub enum PromptType {
    ShapeAndTransform,
    RunLengthEncoding,
}
