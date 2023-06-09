use super::arc_work_model::{Task, PairType};
use super::{Image, HtmlLog, ImageToHTML, ImageSymmetry};

enum InspectPredictedTransform {
    #[allow(dead_code)]
    None,

    #[allow(dead_code)]
    FlipX,

    #[allow(dead_code)]
    FlipY,
}

// static INSPECT_PREDICTED_TRANSFORM: InspectPredictedTransform = InspectPredictedTransform::None;
static INSPECT_PREDICTED_TRANSFORM: InspectPredictedTransform = InspectPredictedTransform::FlipY;

pub struct InspectPredicted;

impl InspectPredicted {
    pub fn inspect(task: &Task, computed_images: &Vec<Image>, status_texts: &Vec<&str>) -> anyhow::Result<()> {

        let f = |image: &Image| -> anyhow::Result<String> {
            let transformed_image: Image = match INSPECT_PREDICTED_TRANSFORM {
                InspectPredictedTransform::None => image.clone(),
                InspectPredictedTransform::FlipX => image.flip_x()?,
                InspectPredictedTransform::FlipY => image.flip_y()?,
            };
            let mut s = String::new();
            s += "<td>";
            s += &transformed_image.to_html();
            s += "</td>";
            Ok(s)
        };

        // Table row with input and row with expected output
        let mut row_input: String = "<tr>".to_string();
        let mut row_output: String = "<tr>".to_string();

        // Traverse the `Train` pairs
        for pair in &task.pairs {
            if pair.pair_type != PairType::Train {
                continue;
            }
            row_input += &f(&pair.input.image)?;
            row_output += &f(&pair.output.image)?;
        }

        // Traverse the `Test` pairs
        for pair in &task.pairs {
            if pair.pair_type != PairType::Test {
                continue;
            }
            row_input += &f(&pair.input.image)?;
            row_output += &f(&pair.output.test_image)?;
        }
        row_input += "<td>Input</td></tr>";
        row_output += "<td>Output</td></tr>";

        // Table row with computed output
        let mut row_predicted: String = "<tr>".to_string();
        for computed_image in computed_images {
            row_predicted += &f(&computed_image)?;
        }
        row_predicted += "<td>Predicted</td></tr>";

        // Table row with status text
        let mut row_status: String = "<tr>".to_string();
        for text in status_texts {
            row_status += &format!("<td>{}</td>", text);
        }
        row_status += "</tr>";

        let html = format!("<h2>{}</h2><table>{}{}{}{}</table>", task.id, row_input, row_output, row_predicted, row_status);
        HtmlLog::html(html);
        Ok(())
    }
}
