use super::{Histogram, Image, ImageHistogram, ImageMask, TaskGraph};
use super::prompt::{PromptSerialize, PromptDeserialize};
use super::arc_work_model::{Task, PairType};
use lazy_static::lazy_static;
use regex::Regex;
use anyhow::{Result, Context};

struct ImageToDictionary;

impl ImageToDictionary {
    /// Creates a python dictionary with x, y coordinates as keys and colors as values.
    /// 
    /// Returns a string like `{(0,0):7,(1,0):7,(2,0):9,(0,1):8,(1,1):7,(2,1):9}`
    fn convert(image: &Image) -> anyhow::Result<String> {
        let mut items = Vec::<String>::new();
        for y in 0..image.height() {
            for x in 0..image.width() {
                let pixel = image.get(x as i32, y as i32).unwrap_or(255);
                items.push(format!("({},{}):{}", x, y, pixel));
            }
        }
        let mut s = String::from("{");
        s += &items.join(",");
        s += "}";
        Ok(s)
    }
}

#[derive(Clone, Debug)]
pub struct PromptPositionSerializer;

impl PromptSerialize for PromptPositionSerializer {
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String> {
        let task: &Task = match &task_graph.task() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };

        let mut rows = Vec::<String>::new();

        rows.push("I'm doing Python experiments.\n\n".to_string());

        rows.push("These are images.".to_string());

        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```python".to_string());
        rows.push("input = {}".to_string());
        rows.push("output = {}".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type == PairType::Test {
                continue;
            }
            if pair_index > 0 {
                rows.push("".to_string());
            }

            {
                let s0: String = ImageToDictionary::convert(&pair.input.image)?;
                let s1: String = format!("input[{}] = {}", pair_index, s0);
                rows.push(s1);
            }

            {
                let s0: String = ImageToDictionary::convert(&pair.output.image)?;
                let s1: String = format!("output[{}] = {}", pair_index, s0);
                rows.push(s1);
            }
        }
        rows.push("```".to_string());
        rows.push("".to_string());
        
        rows.push("# Task A".to_string());
        rows.push("Use at most 50 words.".to_string());
        rows.push("Think step by step.".to_string());
        rows.push("- Write notes about what shapes and patterns you observe.".to_string());
        rows.push("- The output is never the same as the input.".to_string());
        rows.push("\n\n# Task B".to_string());
        rows.push("Use at most 300 words.".to_string());
        rows.push("Include a markdown formatted table with the most important observations about input and output images.".to_string());
        rows.push("The table has three columns: observation name, observation values, comments about the observation.".to_string());
        rows.push("The `observation values` column can contain integers, IDs, yes/no, shape names, absent, decreasing order, and so on.".to_string());
        rows.push("Think step by step.".to_string());
        rows.push("- Count the mass of each layer.".to_string());
        rows.push("- Count how many strongly connected clusters there are inside each layer.".to_string());
        rows.push("- Is mass related to the sorting of layers.".to_string());
        rows.push("- Are there horizontal lines, do they extend edge to edge.".to_string());
        rows.push("- Are there vertical lines, do they extend edge to edge.".to_string());
        rows.push("- Are there shapes such as boxes, L-shape, H-shape, E-shape, Plus-shape, Tetris shapes.".to_string());
        rows.push("- Are there a line connecting two landmarks.".to_string());
        rows.push("- Does shape change color, but preserves their shape, and what may be triggering it.".to_string());
        rows.push("- Does shape move relative x,y.".to_string());
        rows.push("- Boolean operations may happen: xor, and, or.".to_string());
        rows.push("\n\n# Task C".to_string());
        rows.push("Use at most 100 words.".to_string());
        rows.push("Think step by step.".to_string());
        rows.push("What are the actions that converts input to output.".to_string());
        rows.push("\n\n# Task D".to_string());
        rows.push("With the following example input, I want you to predict what the output should be.".to_string());
        rows.push("".to_string());
        rows.push("".to_string());
        rows.push("```python".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            if pair.pair_type == PairType::Train {
                continue;
            }

            {
                let s0: String = ImageToDictionary::convert(&pair.input.image)?;
                let s1: String = format!("input[{}] = {}", pair_index, s0);
                rows.push(s1);
            }
            rows.push("```".to_string());

            rows.push("Print your reasoning before printing the code.".to_string());    
            rows.push("Don't print other markdown code blocks than the code block containing your predictions.".to_string());
            rows.push("\n\nFill your predictions into the following template and replace PREDICT with your predictions.".to_string());
            rows.push("```python".to_string());
            {
                let s1: String = format!("output[{}] = PREDICT", pair_index);
                rows.push(s1);
            }

            // Future experiment:
            // process all the test pairs. Currently it's only 1 test pair.
            break;
        }
        rows.push("```".to_string());

        Ok(rows.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;
    
    #[test]
    fn test_10000_image_to_dictionary() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 9,
            8, 7, 9,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: String = ImageToDictionary::convert(&input).expect("ok");

        // Assert
        let expected = "{(0,0):7,(1,0):7,(2,0):9,(0,1):8,(1,1):7,(2,1):9}";
        assert_eq!(actual, expected);
    }
}
