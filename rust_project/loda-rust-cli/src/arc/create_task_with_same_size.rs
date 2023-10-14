use super::{Image, ImageOverlay};
use super::arc_work_model::{Task, PairType};
use super::arc_json_model::{self, GridFromImage};
use anyhow::Context;

pub struct CreateTaskWithSameSize;

impl CreateTaskWithSameSize {

    /// Make a new task where the input and output sizes are the same.
    ///
    /// The input size and output sizes must be different.
    pub fn create(task: &Task) -> anyhow::Result<Task> {
        if task.is_output_size_same_as_input_size() {
            return Err(anyhow::anyhow!("Cannot process task: {} because it's input and output size is already the same", task.id));
        }

        // find an unused color, that ideally none of the images use, alternatively use the least popular color
        // let mut histogram: Histogram = task.input_histogram_union.clone();
        // histogram.add_histogram(&task.output_histogram_union);
        // let padding_color: u8 = histogram.unused_color().unwrap_or(255).min(9);
        let padding_color: u8 = 10;

        let mut train_pair_vec = Vec::<arc_json_model::TaskPair>::new();
        let mut test_pair_vec = Vec::<arc_json_model::TaskPair>::new();
        for pair in &task.pairs {
            let input: &Image = &pair.input.image;
            let output: &Image;

            match pair.pair_type {
                PairType::Train => {
                    output = &pair.output.image;
                },
                PairType::Test => {
                    output = &pair.output.test_image;
                },
            }

            let width: u8 = input.width().max(output.width());
            let height: u8 = input.height().max(output.height());

            let empty_image = Image::color(width, height, padding_color);
            let input2: Image = empty_image.overlay_with_position(&input, 0, 0)?;
            let output2: Image = empty_image.overlay_with_position(&output, 0, 0)?;

            // Create a Pair instance
            let grid_input: arc_json_model::Grid = arc_json_model::Grid::from_image(&input2);
            let grid_output: arc_json_model::Grid = arc_json_model::Grid::from_image(&output2);
            let task_pair: arc_json_model::TaskPair = arc_json_model::TaskPair::new(grid_input, grid_output);
            match pair.pair_type {
                PairType::Train => {
                    train_pair_vec.push(task_pair);
                },
                PairType::Test => {
                    test_pair_vec.push(task_pair);
                },
            }    
        }

        let task_id_string: String = task.id.clone();
        let task_id = arc_json_model::TaskId::Custom { identifier: task_id_string };

        let json_task = arc_json_model::Task::new(task_id, train_pair_vec, test_pair_vec);

        let task2: Task = Task::try_from(&json_task).context("Unable to convert json_task to work_task")?;
        Ok(task2)
    }
}