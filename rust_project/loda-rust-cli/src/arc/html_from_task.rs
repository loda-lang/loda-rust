use super::arc_work_model;
use super::{InputLabelSet, ActionLabelSet, PropertyInput};
use super::{HtmlLog, ImageToHTML};
use std::collections::HashMap;

pub struct HtmlFromTask;

impl HtmlFromTask {
    fn labelset_to_html(label_set: &ActionLabelSet) -> String {
        let mut label_vec: Vec<String> = label_set.iter().map(|label| format!("{:?}", label)).collect();
        if label_vec.is_empty() {
            return "empty".to_string();
        }
        label_vec.sort();
        label_vec = label_vec.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul>{}</ul>", label_vec.join(""))
    }

    fn input_label_set_to_html(input_label_set: &InputLabelSet) -> String {
        let mut label_vec: Vec<String> = input_label_set.iter().map(|label| format!("{:?}", label)).collect();
        if label_vec.is_empty() {
            return "empty".to_string();
        }
        label_vec.sort();
        label_vec = label_vec.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul>{}</ul>", label_vec.join(""))
    }

    fn input_properties_to_html(input_properties: &HashMap<PropertyInput, u8>) -> String {
        let mut items: Vec<String> = input_properties.iter().map(|(key,value)| format!("{:?} {}", key, value)).collect();
        if items.is_empty() {
            return "empty".to_string();
        }
        items.sort();
        let list_vec: Vec<String> = items.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul>{}</ul>", list_vec.join(""))
    }

    pub fn inspect(task: &arc_work_model::Task) -> anyhow::Result<()> {
        let mut row_title: String = "<tr><td></td>".to_string();
        let mut row_input_image: String = "<tr><td>Input image</td>".to_string();
        let mut row_input_properties: String = "<tr><td>Input properties</td>".to_string();
        let mut row_input_labels: String = "<tr><td>Input labels</td>".to_string();
        let mut row_output_image: String = "<tr><td>Output image</td>".to_string();
        let mut row_action_colors: String = "<tr><td>Action colors</td>".to_string();
        let mut row_action_labels: String = "<tr><td>Action labels</td>".to_string();

        // The current ordering of columns is terrible. train pairs, test pairs, analysis.
        // It's counter intuitive that the analysis comes last.
        // The way the computation takes place is train pairs, analysis, test pairs.
        // TODO: Reorder the columns like this: train pairs, analysis, test pairs.

        for pair in &task.pairs {
            {
                row_title += "<td>";
                let title: &str = match pair.pair_type {
                    arc_work_model::PairType::Train => "Train",
                    arc_work_model::PairType::Test => "Test",
                };
                row_title += title;
                row_title += "</td>";
            }
            {
                row_input_image += "<td>";
                row_input_image += &pair.input.image.to_html();
                row_input_image += "</td>";
            }
            {
                row_input_properties += "<td>";
                row_input_properties += &Self::input_properties_to_html(&pair.input.input_properties);
                row_input_properties += "</td>";
            }
            {
                row_input_labels += "<td>";
                row_input_labels += &Self::input_label_set_to_html(&pair.input.input_label_set);
                row_input_labels += "</td>";
            }
            {
                row_output_image += "<td>";
                row_output_image += &pair.output.image.to_html();
                row_output_image += "</td>";
            }
            {
                row_action_colors += "<td>Removal<br>";
                match pair.removal_histogram.color_image() {
                    Ok(image) => {
                        row_action_colors += &image.to_html();
                    },
                    Err(_) => {
                        row_action_colors += "N/A";
                    }
                }
                row_action_colors += "<br>Insert<br>";
                match pair.insert_histogram.color_image() {
                    Ok(image) => {
                        row_action_colors += &image.to_html();
                    },
                    Err(_) => {
                        row_action_colors += "N/A";
                    }
                }
                row_action_colors += "</td>";
            }
            {
                row_action_labels += "<td>";
                row_action_labels += &Self::labelset_to_html(&pair.action_label_set);
                row_action_labels += "</td>";
            }
        }

        row_title += "<td>Analysis</td>";

        row_input_image += "<td>Union<br>";
        match task.input_histogram_union.color_image() {
            Ok(image) => {
                row_input_image += &image.to_html();
            },
            Err(_) => {
                row_input_image += "N/A";
            }
        }
        row_input_image += "<br><br>Intersection<br>";
        match task.input_histogram_intersection.color_image() {
            Ok(image) => {
                row_input_image += &image.to_html();
            },
            Err(_) => {
                row_input_image += "N/A";
            }
        }
        row_input_image += "</td>";

        row_input_properties += "<td>";
        row_input_properties += &Self::input_properties_to_html(&task.input_properties_intersection);
        row_input_properties += "</td>";

        row_input_labels += "<td>";
        row_input_labels += &Self::input_label_set_to_html(&task.input_label_set_intersection);
        row_input_labels += "</td>";

        row_output_image += "<td>Union<br>";
        match task.output_histogram_union.color_image() {
            Ok(image) => {
                row_output_image += &image.to_html();
            },
            Err(_) => {
                row_output_image += "N/A";
            }
        }
        row_output_image += "<br><br>Intersection<br>";
        match task.output_histogram_intersection.color_image() {
            Ok(image) => {
                row_output_image += &image.to_html();
            },
            Err(_) => {
                row_output_image += "N/A";
            }
        }
        row_output_image += "</td>";

        row_action_colors += "<td>Removal<br>";
        match task.removal_histogram_intersection.color_image() {
            Ok(image) => {
                row_action_colors += &image.to_html();
            },
            Err(_) => {
                row_action_colors += "N/A";
            }
        }
        row_action_colors += "<br>Insert<br>";
        match task.insert_histogram_intersection.color_image() {
            Ok(image) => {
                row_action_colors += &image.to_html();
            },
            Err(_) => {
                row_action_colors += "N/A";
            }
        }
        row_action_colors += "</td>";

        row_action_labels += "<td>";
        row_action_labels += &Self::labelset_to_html(&task.action_label_set_intersection);
        row_action_labels += "</td>";

        row_title += "</tr>";
        row_input_image += "</tr>";
        row_input_properties += "</tr>";
        row_input_labels += "</tr>";
        row_output_image += "</tr>";
        row_action_colors += "</tr>";
        row_action_labels += "</tr>";

        let solution_status: &str;
        if task.occur_in_solutions_csv {
            solution_status = "solved";
        } else {
            solution_status = "UNSOLVED";
        }

        let title: String = format!("{} - {}", task.id, solution_status);

        let html = format!(
            "<h2>{}</h2><p>Output size: {}</p><table>{}{}{}{}{}{}{}</table>",
            title, 
            task.estimated_output_size(),
            row_title,
            row_input_image, 
            row_input_properties, 
            row_input_labels, 
            row_output_image, 
            row_action_colors,
            row_action_labels
        );
        HtmlLog::html(html);
        Ok(())
    }
}
