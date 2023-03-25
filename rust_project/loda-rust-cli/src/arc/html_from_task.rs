use super::arc_work_model;
use super::{InputLabelSet, ActionLabelSet, PropertyInput};
use super::{HtmlLog, ImageToHTML};
use std::collections::HashMap;

pub struct HtmlFromTask {
    row_colgroup: String,
    row_title: String,
    row_input_image: String,
    row_input_properties: String,
    row_input_labels: String,
    row_output_image: String,
    row_action_colors: String,
    row_action_labels: String,
}

impl HtmlFromTask {
    fn new() -> Self {
        Self {
            row_colgroup: "<colgroup><col>".to_string(),
            row_title: "<tr><td></td>".to_string(),
            row_input_image: "<tr><td>Input image</td>".to_string(),
            row_input_properties: "<tr><td>Input properties</td>".to_string(),
            row_input_labels: "<tr><td>Input labels</td>".to_string(),
            row_output_image: "<tr><td>Output image</td>".to_string(),
            row_action_colors: "<tr><td>Action colors</td>".to_string(),
            row_action_labels: "<tr><td>Action labels</td>".to_string(),
        }
    }

    fn labelset_to_html(label_set: &ActionLabelSet) -> String {
        let mut label_vec: Vec<String> = label_set.iter().map(|label| format!("{:?}", label)).collect();
        if label_vec.is_empty() {
            return "empty".to_string();
        }
        label_vec.sort();
        label_vec = label_vec.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul class='without_bullets'>{}</ul>", label_vec.join(""))
    }

    fn input_label_set_to_html(input_label_set: &InputLabelSet) -> String {
        let mut label_vec: Vec<String> = input_label_set.iter().map(|label| format!("{:?}", label)).collect();
        if label_vec.is_empty() {
            return "empty".to_string();
        }
        label_vec.sort();
        label_vec = label_vec.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul class='without_bullets'>{}</ul>", label_vec.join(""))
    }

    fn input_properties_to_html(input_properties: &HashMap<PropertyInput, u8>) -> String {
        let mut items: Vec<String> = input_properties.iter().map(|(key,value)| format!("{:?} {}", key, value)).collect();
        if items.is_empty() {
            return "empty".to_string();
        }
        items.sort();
        let list_vec: Vec<String> = items.iter().map(|label| format!("<li>{}</li>", label)).collect();
        format!("<ul class='without_bullets'>{}</ul>", list_vec.join(""))
    }

    fn push_pair(&mut self, pair: &arc_work_model::Pair) -> anyhow::Result<()> {
        {
            match pair.pair_type {
                arc_work_model::PairType::Train => {
                    self.row_colgroup += "<col class='arc_column_pair_train'>";
                },
                arc_work_model::PairType::Test => {
                    self.row_colgroup += "<col class='arc_column_pair_test'>";
                },
            };
        }
        {
            self.row_title += "<td>";
            let title: &str = match pair.pair_type {
                arc_work_model::PairType::Train => "Train",
                arc_work_model::PairType::Test => "Test",
            };
            self.row_title += title;
            self.row_title += "</td>";
        }
        {
            self.row_input_image += "<td>";
            self.row_input_image += &pair.input.image.to_html();
            self.row_input_image += "</td>";
        }
        {
            self.row_input_properties += "<td>";
            self.row_input_properties += &Self::input_properties_to_html(&pair.input.input_properties);
            self.row_input_properties += "</td>";
        }
        {
            self.row_input_labels += "<td>";
            self.row_input_labels += &Self::input_label_set_to_html(&pair.input.input_label_set);
            self.row_input_labels += "</td>";
        }
        {
            self.row_output_image += "<td>";
            self.row_output_image += &pair.output.image.to_html();
            self.row_output_image += "</td>";
        }
        {
            self.row_action_colors += "<td>Removal<br>";
            match pair.removal_histogram.color_image() {
                Ok(image) => {
                    self.row_action_colors += &image.to_html();
                },
                Err(_) => {
                    self.row_action_colors += "N/A";
                }
            }
            self.row_action_colors += "<br>Insert<br>";
            match pair.insert_histogram.color_image() {
                Ok(image) => {
                    self.row_action_colors += &image.to_html();
                },
                Err(_) => {
                    self.row_action_colors += "N/A";
                }
            }
            self.row_action_colors += "</td>";
        }
        {
            self.row_action_labels += "<td>";
            self.row_action_labels += &Self::labelset_to_html(&pair.action_label_set);
            self.row_action_labels += "</td>";
        }
        Ok(())
    }

    fn push_column_analysis(&mut self, task: &arc_work_model::Task) -> anyhow::Result<()> {
        let classname: &str = "class='arc_column_analysis'";
        let td_begin: String = format!("<td {}>", classname);
        self.row_colgroup += &format!("<col {}>", classname);

        self.row_title += &td_begin;
        self.row_title += "Analysis</td>";

        self.row_input_image += &td_begin;
        self.row_input_image += "Union<br>";
        match task.input_histogram_union.color_image() {
            Ok(image) => {
                self.row_input_image += &image.to_html();
            },
            Err(_) => {
                self.row_input_image += "N/A";
            }
        }
        self.row_input_image += "<br><br>Intersection<br>";
        match task.input_histogram_intersection.color_image() {
            Ok(image) => {
                self.row_input_image += &image.to_html();
            },
            Err(_) => {
                self.row_input_image += "N/A";
            }
        }
        self.row_input_image += "</td>";

        self.row_input_properties += &td_begin;
        self.row_input_properties += &Self::input_properties_to_html(&task.input_properties_intersection);
        self.row_input_properties += "</td>";

        self.row_input_labels += &td_begin;
        self.row_input_labels += &Self::input_label_set_to_html(&task.input_label_set_intersection);
        self.row_input_labels += "</td>";

        self.row_output_image += &td_begin;
        self.row_output_image += "Union<br>";
        match task.output_histogram_union.color_image() {
            Ok(image) => {
                self.row_output_image += &image.to_html();
            },
            Err(_) => {
                self.row_output_image += "N/A";
            }
        }
        self.row_output_image += "<br><br>Intersection<br>";
        match task.output_histogram_intersection.color_image() {
            Ok(image) => {
                self.row_output_image += &image.to_html();
            },
            Err(_) => {
                self.row_output_image += "N/A";
            }
        }
        self.row_output_image += "</td>";

        self.row_action_colors += &td_begin;
        self.row_action_colors += "Removal<br>";
        match task.removal_histogram_intersection.color_image() {
            Ok(image) => {
                self.row_action_colors += &image.to_html();
            },
            Err(_) => {
                self.row_action_colors += "N/A";
            }
        }
        self.row_action_colors += "<br>Insert<br>";
        match task.insert_histogram_intersection.color_image() {
            Ok(image) => {
                self.row_action_colors += &image.to_html();
            },
            Err(_) => {
                self.row_action_colors += "N/A";
            }
        }
        self.row_action_colors += "</td>";

        self.row_action_labels += &td_begin;
        self.row_action_labels += &Self::labelset_to_html(&task.action_label_set_intersection);
        self.row_action_labels += "</td>";
        Ok(())
    }

    fn end_of_row(&mut self) {
        self.row_colgroup += "</colgroup>";
        self.row_title += "</tr>";
        self.row_input_image += "</tr>";
        self.row_input_properties += "</tr>";
        self.row_input_labels += "</tr>";
        self.row_output_image += "</tr>";
        self.row_action_colors += "</tr>";
        self.row_action_labels += "</tr>";
    }

    fn to_html(&self, task: &arc_work_model::Task) -> String {
        let solution_status: &str;
        if task.occur_in_solutions_csv {
            solution_status = "solved";
        } else {
            solution_status = "UNSOLVED";
        }

        let title: String = format!("{} - {}", task.id, solution_status);

        let thead: String = format!("<thead>{}</thead>", self.row_title);
        let tbody: String = format!(
            "<tbody>{}{}{}{}{}{}</tbody>",
            self.row_input_image, 
            self.row_input_properties, 
            self.row_input_labels, 
            self.row_output_image, 
            self.row_action_colors,
            self.row_action_labels
        );

        let table: String = format!(
            "<table class='arc_work_model_task'>{}{}{}</table>",
            self.row_colgroup,
            thead,
            tbody
        );

        let html = format!(
            "<h2>{}</h2><p>Output size: {}</p>{}",
            title, 
            task.estimated_output_size(),
            table,
        );
        html
    }

    pub fn inspect(task: &arc_work_model::Task) -> anyhow::Result<()> {
        let mut instance = Self::new();

        // The current ordering of columns is terrible. train pairs, test pairs, analysis.
        // It's counter intuitive that the analysis comes last.
        // The way the computation takes place is train pairs, analysis, test pairs.
        // TODO: Reorder the columns like this: train pairs, analysis, test pairs.

        for pair in &task.pairs {
            if pair.pair_type != arc_work_model::PairType::Train {
                continue;
            }
            instance.push_pair(pair)?;
        }

        instance.push_column_analysis(task)?;

        for pair in &task.pairs {
            if pair.pair_type != arc_work_model::PairType::Test {
                continue;
            }
            instance.push_pair(pair)?;
        }

        instance.end_of_row();

        let html: String = instance.to_html(task);
        HtmlLog::html(html);
        Ok(())
    }
}
