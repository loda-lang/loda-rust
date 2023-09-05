use super::{Image, TaskGraph};
use super::prompt::PromptSerialize;
use super::arc_work_model::{Task, PairType};

struct ImageToText;

impl ImageToText {
    /// Creates a compact string representation of an ARC image.
    /// 
    /// If `include_size` is false, then there is no width and height info in the dictionary.
    /// Returns a string like `008000700,008000700,888888288,008000700,008000700,008000700,772777777,008000700,008000700`
    /// 
    /// If `include_size` is true, then it will include the width and height of the image, like this
    /// `width9,height9,008000700,008000700,888888288,008000700,008000700,008000700,772777777,008000700,008000700`
    fn convert(image: &Image, include_size: bool) -> anyhow::Result<String> {
        let mut items = Vec::<String>::new();
        if include_size {
            items.push(format!("width{}", image.width()));
            items.push(format!("height{}", image.height()));
        }
        for y in 0..image.height() {
            let mut s = String::new();
            for x in 0..image.width() {
                let pixel = image.get(x as i32, y as i32).unwrap_or(255);
                s += &format!("{}", pixel);
            }
            items.push(s);
        }
        Ok(items.join(","))
    }
}

#[derive(Clone, Debug)]
pub struct PromptCompactSerializer;

impl PromptSerialize for PromptCompactSerializer {
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String> {
        let task: &Task = match &task_graph.task() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };

        let include_size: bool = true;

        let mut rows = Vec::<String>::new();

        rows.push("Hi, I'm doing Python experiments.\n\n".to_string());

        rows.push("These are images.".to_string());

        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```python".to_string());
        rows.push("input = {}".to_string());
        rows.push("output = {}".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            rows.push(format!("# Group{}", pair_index));

            {
                let s0: String = ImageToText::convert(&pair.input.image, include_size)?;
                let s1: String = format!("input[{}] = '{}'", pair_index, s0);
                rows.push(s1);
            }

            match pair.pair_type {
                PairType::Train => {
                    let s0: String = ImageToText::convert(&pair.output.image, include_size)?;
                    let s1: String = format!("output[{}] = '{}'", pair_index, s0);
                    rows.push(s1);
                },
                PairType::Test => {
                    let s1: String = format!("output[{}] = 'PREDICT'", pair_index);
                    rows.push(s1);
                }
            }
        }
        rows.push("```".to_string());
        rows.push("".to_string());
        

        Ok(rows.join("\n"))
    }
}
