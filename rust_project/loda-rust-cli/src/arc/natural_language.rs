use super::prompt::PromptSerialize;
use super::{ShapeTransformation, Image, ImageToHTML, ImageSize, ShapeType, NodeData, GraphNodeDataEdgeData, TaskGraph, ImageType, PixelConnectivity};
use super::arc_work_model::{Task, PairType};
use std::collections::HashSet;
use regex::Regex;
use lazy_static::lazy_static;
use petgraph::{stable_graph::NodeIndex, visit::EdgeRef};

lazy_static! {
    /// Extract the bounding box from strings like: `ignore_t3_l7_b7_r11_ignore`
    static ref EXTRACT_TLBR: Regex = Regex::new(
        "t(-?\\d+)_l(-?\\d+)_b(-?\\d+)_r(-?\\d+)"
    ).unwrap();

    /// Extract the `id` prefixed data from strings like: `ignore_idP3d53ef_ignore`
    static ref EXTRACT_ID: Regex = Regex::new(
        "id([A-Za-z0-9]{1,10})"
    ).unwrap();

    /// Extract the `shape` prefixed data from strings like: `ignore_shapeRectangle_ignore`
    static ref EXTRACT_SHAPE: Regex = Regex::new(
        "shape([A-Za-z0-9]{1,30})"
    ).unwrap();

    /// Extract the `m` prefixed data from strings like: `ignore_m42_ignore`
    static ref EXTRACT_MASS: Regex = Regex::new(
        "m(\\d+)"
    ).unwrap();

    /// Extract the `transform(...)` values from strings like: `transform(rot90_rot270_flip90_flip270)`
    static ref EXTRACT_TRANSFORM: Regex = Regex::new(
        "transform[(]([a-z0-9_]{1,100})[)]"
    ).unwrap();

    /// Extract width=4 and height=3 from strings like: `ignore_width4_height3_ignore`
    static ref EXTRACT_WIDTH_HEIGHT: Regex = Regex::new(
        "width(\\d+)_height(\\d+)"
    ).unwrap();
}

const MOCK_REPLY1: &str = r#"
From the examples given, it appears the transformation from input to output operates as follows:

1. Each object in the input, regardless of its original shape, mass, and transformation properties, is reduced to a 1x1 rectangle in the output.

2. The mass of each object in the output is always 1, irrespective of the mass of the input object.

3. The transformation applied to the output objects is always "all".

4. The bounding coordinates of the output objects are restructured such that the top, left, bottom, and right parameters describe a 1x1 rectangle. 

5. The ordering of the objects in the output seems to be determined by the top coordinate (t), from the lowest to the highest.

Given these rules, the predicted output for Example 4 should be as follows:

```prolog
% Example 4 input grid_width12_height12
object(input4_idP48kmo7_t11_l6_b11_r6_w1_h1_m1_shapeRectangle_scalex1_scaley1, transform(all)).
object(input4_idP48kmo7_t9_l7_b12_r9_w3_h4_m6_shapeUnclassified_scalex1_scaley1, transform(rot90_rot270)).
object(input4_idP33ffe7_t2_l2_b3_r3_w2_h2_m3_shapeL_scalex1_scaley1, transform(rot90_flip)).
object(input4_idP3d53ef_t6_l4_b7_r6_w3_h2_m5_shapeL_scaleUnknown, transform(rot90_flip)).

% Example 4 output grid_width1_height3
object(output4_idP33ffe7_t1_l1_b1_r1_w1_h1_m1_shapeRectangle_scalex1_scaley1, transform(all)).
object(output4_idP3d53ef_t2_l1_b2_r1_w1_h1_m1_shapeRectangle_scalex1_scaley1, transform(all)).
object(output4_idP48kmo7_t3_l1_b3_r1_w1_h1_m1_shapeRectangle_scalex1_scaley1, transform(all)).
```

Note: Even though there are two objects with the id "idP48kmo7" in the input, only one of them is represented in the output. The one with the lower 't' value is represented following the sorting rule.
"#;


/// XY coordinates for Top-Left corner and Bottom-Right corner. Aka. `TLBR`.
#[derive(Clone, Debug)]
pub struct TLBR {
    pub raw_top: i8,
    pub raw_left: i8,
    pub raw_bottom: i8,
    pub raw_right: i8,
    pub top: i8,
    pub left: i8,
    pub bottom: i8,
    pub right: i8,
}

impl TryFrom<&str> for TLBR {
    type Error = anyhow::Error;

    /// Extract the bounding box from strings like: `ignore_t3_l7_b7_r11_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_TLBR;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract TLBR from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
        let capture3: &str = captures.get(3).map_or("", |m| m.as_str());
        let capture4: &str = captures.get(4).map_or("", |m| m.as_str());
        let raw_top = capture1.parse::<i8>()?;
        let raw_left = capture2.parse::<i8>()?;
        let raw_bottom = capture3.parse::<i8>()?;
        let raw_right = capture4.parse::<i8>()?;
        let instance = Self {
            raw_top,
            raw_left,
            raw_bottom,
            raw_right,
            top: raw_top.min(raw_bottom),
            left: raw_left.min(raw_right),
            bottom: raw_top.max(raw_bottom),
            right: raw_left.max(raw_right),
        };
        Ok(instance)
    }
}

/// The `FieldID` holds the obfuscated color value.
///
/// The non-obfuscated color value didn't work with the language models I have tried. 
/// Often the language model would interpret the color as an integer value or RGB value.
/// In ARC the color is an opaque value that has no meaning other than being a symbol identifier,
/// that uniquely identifies each color.
#[derive(Clone, Debug)]
pub struct FieldId {
    pub name: String,
    pub value: u8,
}

impl FieldId {
    pub fn id_from_value(value: u8) -> String {
        let name: String = Self::name_from_value(value);
        format!("id{}", name)
    }

    pub fn name_from_value(value: u8) -> String {
        let name: &str = match value {
            0 => "P2a5e30",
            1 => "P3d53ef",
            2 => "Pfe7a8k",
            3 => "P33ffe7",
            4 => "P989a7f",
            5 => "Pj8kdf4",
            6 => "P48kmo7",
            7 => "P847fa3",
            8 => "Pz7ea0g",
            9 => "P03hft3",
            _ => "Unknown"
        };
        name.to_string()
    }

    pub fn value_from_name(name: &str) -> Option<u8> {
        let value: u8 = match name {
            "P2a5e30" => 0,
            "P3d53ef" => 1,
            "Pfe7a8k" => 2,
            "P33ffe7" => 3,
            "P989a7f" => 4,
            "Pj8kdf4" => 5,
            "P48kmo7" => 6,
            "P847fa3" => 7,
            "Pz7ea0g" => 8,
            "P03hft3" => 9,
            _ => {
                return None;
            }
        };
        Some(value)
    }
}

impl TryFrom<&str> for FieldId {
    type Error = anyhow::Error;

    /// Extract the `id` prefixed data from strings like: `ignore_idP3d53ef_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_ID;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract ID from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let value: u8 = match Self::value_from_name(capture1) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract ID from string. Unrecognized value: '{}'", capture1);
            }
        };
        let instance = Self {
            name: capture1.to_string(),
            value,
        };
        Ok(instance)
    }
}

/// The `FieldMass` holds the mass of the object.
#[derive(Clone, Debug)]
pub struct FieldMass {
    /// The max image size is 255x255, so it fits in a u16.
    pub mass: u16,
}

impl TryFrom<&str> for FieldMass {
    type Error = anyhow::Error;

    /// Extract the `m` prefixed data from strings like: `ignore_m42_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_MASS;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract MASS from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let mass = capture1.parse::<u16>()?;
        let instance = Self {
            mass,
        };
        Ok(instance)
    }
}

/// The `FieldShape` holds the shape type.
#[derive(Clone, Debug)]
pub struct FieldShape {
    pub shape_name: String,
}

impl TryFrom<&str> for FieldShape {
    type Error = anyhow::Error;

    /// Extract the `shape` prefixed data from strings like: `ignore_shapeRectangle_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_SHAPE;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract SHAPE from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let instance = Self {
            shape_name: capture1.to_string(),
        };
        Ok(instance)
    }
}

/// The `FieldTransform` holds the transformations of the object.
#[derive(Clone, Debug)]
pub struct FieldTransform {
    pub raw: String,
    pub transformations: HashSet<ShapeTransformation>,
}

impl FieldTransform {
    fn natural_language_name(transformation: &ShapeTransformation) -> &'static str {
        match transformation {
            ShapeTransformation::Normal => "rot0",
            ShapeTransformation::RotateCw90 => "rot90",
            ShapeTransformation::RotateCw180 => "rot180",
            ShapeTransformation::RotateCw270 => "rot270",
            ShapeTransformation::FlipX => "flip",
            ShapeTransformation::FlipXRotateCw90 => "flip90",
            ShapeTransformation::FlipXRotateCw180 => "flip180",
            ShapeTransformation::FlipXRotateCw270 => "flip270",
        }
    }
}

impl TryFrom<&str> for FieldTransform {
    type Error = anyhow::Error;

    /// Extract the `transform` data from strings like: `transform(rot90_rot270)`
    /// Split on underscore `_` to get the individual transformations.
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_TRANSFORM;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract TRANSFORM from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let raw: String = capture1.to_string();

        let mut transformations = HashSet::<ShapeTransformation>::new();
        if capture1 == "all" {
            transformations = ShapeTransformation::all();
        } else {
            // Split on underscore `_` to get the individual transformations.
            for item in capture1.split("_") {
                let transformation: ShapeTransformation = match item {
                    "rot0" => ShapeTransformation::Normal,
                    "rot90" => ShapeTransformation::RotateCw90,
                    "rot180" => ShapeTransformation::RotateCw180,
                    "rot270" => ShapeTransformation::RotateCw270,
                    "flip" => ShapeTransformation::FlipX,
                    "flip90" => ShapeTransformation::FlipXRotateCw90,
                    "flip180" => ShapeTransformation::FlipXRotateCw180,
                    "flip270" => ShapeTransformation::FlipXRotateCw270,
                    _ => {
                        anyhow::bail!("Unable to parse TRANSFORM from string. The item '{}' is not recognized.", item);
                    }
                };
                transformations.insert(transformation);
            }
        }
        if transformations.is_empty() {
            anyhow::bail!("Unable to parse TRANSFORM from string. The transformations set is empty");
        }

        let instance = Self {
            raw,
            transformations,
        };
        Ok(instance)
    }
}

/// Extract width and height parameters.
#[derive(Clone, Debug)]
pub struct FieldWidthHeight {
    pub width: u8,
    pub height: u8,
}

impl TryFrom<&str> for FieldWidthHeight {
    type Error = anyhow::Error;

    /// Extract width=4 and height=3 from a string like: `ignore_width4_height3_ignore`
    fn try_from(singleline_text: &str) -> Result<Self, Self::Error> {
        let re = &EXTRACT_WIDTH_HEIGHT;
        let captures = match re.captures(&singleline_text) {
            Some(value) => value,
            None => {
                anyhow::bail!("Unable to extract TLBR from string");
            }
        };
        let capture1: &str = captures.get(1).map_or("", |m| m.as_str());
        let capture2: &str = captures.get(2).map_or("", |m| m.as_str());
        let width = capture1.parse::<u8>()?;
        let height = capture2.parse::<u8>()?;
        if width == 0 || height == 0 {
            anyhow::bail!("Invalid width or height. Must be greater than zero");
        }
        if width > 30 || height > 30 {
            anyhow::bail!("Invalid width or height. Must be less than or equal to 30");
        }
        let instance = Self {
            width,
            height,
        };
        Ok(instance)
    }
}

#[derive(Clone, Debug)]
pub struct NaturalLanguage {
    pub lines: Vec<String>,
    pub width_height: Option<FieldWidthHeight>,
}

impl NaturalLanguage {
    #[allow(dead_code)]
    pub fn reply_example1() -> String {
        MOCK_REPLY1.to_string()
    }

    fn interpret_line(line_index: usize, line: &str) {
        println!("line: {}", line_index);
        if let Ok(id) = FieldId::try_from(line) {
            println!("id: {:?}", id);
        }
        if let Ok(tlbr) = TLBR::try_from(line) {
            println!("tlbr: {:?}", tlbr);
        }
        if let Ok(shape) = FieldShape::try_from(line) {
            println!("shape: {:?}", shape);
        }
        if let Ok(mass) = FieldMass::try_from(line) {
            println!("mass: {:?}", mass);
        }
        if let Ok(transform) = FieldTransform::try_from(line) {
            println!("transform: {:?}", transform);
        }
    }

    #[allow(dead_code)]
    fn interpret(&self) {
        for (line_index, line) in self.lines.iter().enumerate() {
            Self::interpret_line(line_index, line);
        }
    }

    fn interpret_line_and_draw(_line_index: usize, line: &str, image: &mut Image) -> anyhow::Result<()> {
        // Color from obfuscated color name
        let id = FieldId::try_from(line)?;
        let color: u8 = id.value;

        // Coordinates for bounding box
        let tlbr = TLBR::try_from(line)?;
        // println!("tlbr: {:?}", tlbr);

        let object_x: i32 = tlbr.left as i32 - 1;
        let object_y: i32 = tlbr.top as i32 - 1;
        let object_width: i32 = tlbr.right as i32 - tlbr.left as i32 + 1;
        let object_height: i32 = tlbr.bottom as i32 - tlbr.top as i32 + 1;

        if object_width < 0 || object_height < 0 {
            anyhow::bail!("Invalid width or height");
        }

        let mut _count_draw: usize = 0;
        for y in 0..image.height() {
            for x in 0..image.width() {
                let xx: i32 = x as i32;
                let yy: i32 = y as i32;

                if xx >= object_x && xx < object_x + object_width && yy >= object_y && yy < object_y + object_height {
                    image.set(xx, yy, color);
                    _count_draw += 1;
                }
            }
        }
        // println!("count_draw: {}", count_draw);
        
        Ok(())
    }

    fn interpret_and_draw(&self, image: &mut Image) {
        for (line_index, line) in self.lines.iter().enumerate() {
            match Self::interpret_line_and_draw(line_index, line, image) {
                Ok(_) => {},
                Err(error) => {
                    println!("Error: {}", error);
                }
            }
        }
    }

    pub fn to_html(&self) -> String {
        let mut image = Image::zero(30, 30);
        if let Some(width_height) = &self.width_height {
            image = Image::zero(width_height.width, width_height.height);
        }

        self.interpret_and_draw(&mut image);

        let mut s = String::new();
        s += &image.to_html();
        s
    }
}

impl TryFrom<&str> for NaturalLanguage {
    type Error = anyhow::Error;

    /// Extract the interesting parts from the prompt response.
    /// 
    /// The response is supposed to contain a markdown formatted text
    /// with three backticks to mark the beginning and end of a code block.
    /// The code block of interest starts with `prolog`.
    /// 
    /// Within the `prolog` code block, there is supposed to be
    /// a list of `object(input...` and `object(output...` lines.
    /// 
    /// It's the `object(output...` lines that are of interest,
    /// that gets extracted.
    fn try_from(multiline_text: &str) -> Result<Self, Self::Error> {
        let mut lines_with_prefix = Vec::<String>::new();
        let mut inside_code_block = false;
        let mut count_unrecognized_inside_code_block: usize = 0;
        let mut count_code_block: usize = 0;
        let mut found_width_height: Option<FieldWidthHeight> = None;
        for line in multiline_text.split("\n") {
            let trimmed_line: &str = line.trim();
            if trimmed_line.contains("```prolog") {
                if count_code_block == 0 {
                    inside_code_block = true;
                }
                count_code_block += 1;
                continue;
            }
            if !inside_code_block {
                continue;
            }
            if trimmed_line == "```" {
                inside_code_block = false;
                continue;
            }
            if trimmed_line.is_empty() {
                continue;
            }
            if trimmed_line.starts_with("%") {
                if trimmed_line.contains("output grid") {
                    let width_height: FieldWidthHeight = FieldWidthHeight::try_from(trimmed_line)?;
                    found_width_height = Some(width_height);
                }
                continue;
            }
            if trimmed_line.starts_with("object(input") {
                continue;
            }
            if trimmed_line.starts_with("object(output") {
                lines_with_prefix.push(line.to_string());
                continue;
            }
            count_unrecognized_inside_code_block += 1;
        }
        if count_code_block == 0 {
            anyhow::bail!("No code block found. Expected a code block starting with 3 backticks and prolog.");
        }
        if count_code_block >= 2 {
            anyhow::bail!("Multiple code blocks found. Expected just one code block starting with 3 backticks and prolog.");
        }
        if count_unrecognized_inside_code_block > 0 {
            anyhow::bail!("{} unrecognized lines inside the code block", count_unrecognized_inside_code_block);
        }
        let instance = Self {
            lines: lines_with_prefix,
            width_height: found_width_height,
        };
        Ok(instance)
    }
}

#[derive(Clone, Debug)]
pub struct NaturalLanguageSerializer {
    connectivity: PixelConnectivity
}

impl NaturalLanguageSerializer {
    pub fn new_connectivity4() -> Self {
        Self {
            connectivity: PixelConnectivity::Connectivity4,
        }
    }

    pub fn new_connectivity8() -> Self {
        Self {
            connectivity: PixelConnectivity::Connectivity8,
        }
    }

    fn natural_language_of_object(graph: &GraphNodeDataEdgeData, object_nodeindex: NodeIndex) -> anyhow::Result<String> {
        let mut found_position_x: Option<u8> = None;
        let mut found_position_y: Option<u8> = None;
        let mut found_shapesize: Option<ImageSize> = None;
        let mut found_mass: Option<u16> = None;
        let mut found_color: Option<u8> = None;
        let mut found_shapetype: Option<ShapeType> = None;
        let mut found_shapetransformations: Option<String> = None;
        let mut found_shapescale: Option<String> = None;
        for edge in graph.edges(object_nodeindex) {
            let node_index: NodeIndex = edge.target();
            match &graph[node_index] {
                NodeData::PositionX { x } => {
                    found_position_x = Some(*x);
                },
                NodeData::PositionY { y } => {
                    found_position_y = Some(*y);
                },
                NodeData::ShapeSize { width, height } => {
                    found_shapesize = Some(ImageSize::new(*width, *height));
                },
                NodeData::Mass { mass } => {
                    found_mass = Some(*mass);
                },
                NodeData::Color { color } => {
                    found_color = Some(*color);
                },
                NodeData::ShapeType { shape_type } => {
                    found_shapetype = Some(*shape_type);
                },
                NodeData::ShapeTransformations { transformations } => {
                    if transformations.len() == 8 {
                        found_shapetransformations = Some("all".to_string());
                        continue;
                    }
                    let items: Vec<String> = transformations.iter().map(|t| FieldTransform::natural_language_name(t).to_string()).collect::<Vec<String>>();
                    let s = format!("{}", items.join("_"));
                    found_shapetransformations = Some(s);
                },
                NodeData::ShapeScale { x, y } => {
                    let s = format!("scalex{}_scaley{}", x, y);
                    found_shapescale = Some(s);
            },
                _ => {}
            }
        }
        let mut items = Vec::<String>::new();
        if let Some(color) = found_color {
            let obfuscated_color: String = FieldId::id_from_value(color);
            items.push(obfuscated_color);
        }
        // if let Some(position_x) = found_position_x {
            // items.push(format!("x{}", position_x));
            // items.push(format!("{}", position_x + 1));
        // }
        // if let Some(position_y) = found_position_y {
            // items.push(format!("y{}", position_y));
            // items.push(format!("{}", position_y + 1));
        // }
        // if let Some(size) = found_shapesize {
            // items.push(format!("width{}_height{}", size.width, size.height));
            // let x: i32 = size.width as i32 - 1;
            // let y: i32 = size.height as i32 - 1;
            // items.push(format!("{}_{}", x, y));
            // items.push(format!("{}_{}", size.width, size.height));
        // }
        match (found_position_x, found_position_y, found_shapesize) {
            (Some(x), Some(y), Some(size)) => {
                // items.push(format!("coord{}_{}_{}_{}", x + 1, y + 1, x + size.width, y + size.height));
                // items.push(format!("tl{}_{}_br{}_{}", x + 1, y + 1, x + size.width, y + size.height));
                items.push(format!("t{}_l{}_b{}_r{}", y + 1, x + 1, y + size.height, x + size.width));
                items.push(format!("w{}_h{}", size.width, size.height));
            },
            _ => {}
        }
        if let Some(mass) = found_mass {
            items.push(format!("m{}", mass));
        }
        if let Some(shapetype) = found_shapetype {
            items.push(format!("shape{:?}", shapetype));
        }
        if let Some(shapescale) = found_shapescale {
            items.push(shapescale);
        } else {
            items.push(format!("scaleUnknown"));
        }
        let mut s = String::new();
        s += &items.join("_");
        if let Some(shapetransformations) = found_shapetransformations {
            s += &format!(", transform({})", shapetransformations);
        }
        Ok(s)
    }
}

impl PromptSerialize for NaturalLanguageSerializer {
    /// Convert the `TaskGraph` into a prompt for a language model to solve.
    /// 
    /// Known problem: It can only ask prompt about the first `test` pair.
    /// The tasks that have more than one `test` pair, will not create prompts for the remaining `test` pairs.
    /// 
    /// Known problem: Assumes that the background color is 0, and treat it as transparent.
    /// Tasks where the background color is not 0, will have to be handled in a different way.
    /// 
    /// Known problem: Generates lots of text. For tasks that have many objects, 
    /// the prompt may be too long for the language model to process.
    fn to_prompt(&self, task_graph: &TaskGraph) -> anyhow::Result<String> {
        let connectivity: PixelConnectivity = self.connectivity;
        
        let task: &Task = match &task_graph.task() {
            Some(value) => value,
            None => {
                return Err(anyhow::anyhow!("graph is not initialized with a task"));
            }
        };
        let graph: &GraphNodeDataEdgeData = task_graph.graph();

        let mut rows = Vec::<String>::new();

        rows.push("I'm doing Prolog experiments.\n\n".to_string());
        // rows.push("I'm doing Prolog experiments with grids.\n\n".to_string());
        // rows.push("I'm doing Prolog experiments and you are going to emit a prolog program with the predicted output objects.\n\n".to_string());
        // rows.push("You are a Prolog expert developer.\n\n".to_string());
        
        rows.push("The grid is 1-indexed.\n\n".to_string());
        // rows.push("The grid is 1-indexed and does allow negative indices and coordinates outside the grid size. The top left corner has coordinate x=1, y=1.\n\n".to_string());

        // rows.push("The coordinates are topleftx_toplefty_bottomrightx_bottomrighty.\n\n".to_string());
        // rows.push("The coordinates are provided as tlX_Y_brX_Y where tl is topleft and br is bottomright.\n\n".to_string());
        // rows.push("The coordinates are TLBR formatted, tY_lX_bY_rX where t=top l=left b=bottom r=right.\n\n".to_string());
        rows.push("Top-Left Bottom-Right (TLBR) is used for object bounding boxes, like tY_lX_bY_rX where t=top l=left b=bottom r=right.\n\n".to_string());
        // rows.push("Top-Left Bottom-Right (TLBR) is used for object rectangles, like tY_lX_bY_rX where t=top l=left b=bottom r=right.\n\n".to_string());

        rows.push("The width of the object bounding box has a 'w' prefix, like 'w5' is width=5.".to_string());
        rows.push("The height of the object bounding box has a 'h' prefix, like 'h3' is height=3.\n".to_string());
        // rows.push("The x coordinates has this relationship: left + width - 1 = right.".to_string());
        // rows.push("The y coordinates has this relationship: top + height - 1 = bottom.\n\n".to_string());

        rows.push("The number of solid pixels in the object has a 'm' prefix, like 'm12' is mass=12.\n".to_string());

        // rows.push("The `id` prefixed text has no integer value and should not be considered. The number of unique IDs may be relevant. The mass of each ID is sometimes preserved in the output.".to_string());
        rows.push("The `id` prefixed text has no integer value and should not be considered for sorting. The number of unique IDs may be relevant. The total mass of certain IDs is sometimes preserved in the output.".to_string());

        // rows.push("No rectangle overlap with other rectangles.\n\n".to_string());

        // rows.push("There number of output objects may be different than the input objects.".to_string());
        // rows.push("The output objects may have to be sorted by coordinates or mass or some other property.".to_string());
        // rows.push("The output objects have experience gravity towards other objects. An object with a specific `id` may attract other objects.".to_string());

        rows.push("```prolog".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let pair_index_u8: u8 = pair_index.min(255) as u8;
            let natural_language_pair_index: String = format!("{}", pair_index + 1);
            if pair.pair_type == PairType::Test {
                continue;
            }
            if pair_index > 0 {
                rows.push("".to_string());
            }

            {
                let size: ImageSize = pair.input.image.size();
                let s = format!("% Example {} input grid_width{}_height{}", natural_language_pair_index, size.width, size.height);
                rows.push(s);
            }

            {
                let object_nodeindex_vec: Vec::<NodeIndex> = task_graph.get_object_nodeindex_vec(pair_index_u8, ImageType::Input, connectivity)?;
                for object_nodeindex in &object_nodeindex_vec {
                    let s0: String = NaturalLanguageSerializer::natural_language_of_object(graph, *object_nodeindex)?;
                    let s1: String = format!("object(input{}_{}).", natural_language_pair_index, s0);
                    rows.push(s1);
                }        
            }

            rows.push("".to_string());
            {
                let size: ImageSize = pair.output.image.size();
                let s = format!("% Example {} output grid_width{}_height{}", natural_language_pair_index, size.width, size.height);
                rows.push(s);
            }

            {
                let object_nodeindex_vec: Vec::<NodeIndex> = task_graph.get_object_nodeindex_vec(pair_index_u8, ImageType::Output, connectivity)?;
                for object_nodeindex in &object_nodeindex_vec {
                    let s0: String = NaturalLanguageSerializer::natural_language_of_object(graph, *object_nodeindex)?;
                    let s1: String = format!("object(output{}_{}).", natural_language_pair_index, s0);
                    rows.push(s1);
                }        
            }
        }
        rows.push("```".to_string());
        rows.push("".to_string());
        rows.push("".to_string());
        // rows.push("\n\nWhat example has the biggest number of columns?".to_string());
        // rows.push("\n\nWhat are the transformations across all the examples, that goes from the input to the output?".to_string());
        // rows.push("The shapeRectangle is solid and cannot overlap with other objects. Create more shapeRectangle objects in order to ensure no overlap.".to_string());
        // rows.push("There number of output objects may be different than the input objects.".to_string());

        // rows.push("Assumptions:".to_string());
        // rows.push("- Assume that shapeRectangle is solid and have no holes.".to_string());
        // rows.push("- Assume that shapeBox has 1 rectangular hole.".to_string());
        // rows.push("- Assume that shapeBoxWithTwoHoles has a middle separator and 2 rectangular holes.".to_string());
        // rows.push("".to_string());

        rows.push("There number of output objects can be different than the input objects. Also consider the rules with clockwise rotation.".to_string());
        rows.push("Check if a condition is satisfied only for objects with a certain shape.".to_string());

        rows.push("A shape can occlude another shape, so shapeL may appear as shapeRectangle. Sometimes it's the occluded object that gets transformed.".to_string());
        rows.push("Consider both euclidian distance and manhatten distance between objects.".to_string());
        rows.push("Check how much an object moves relative x, y.".to_string());
        rows.push("Check how IDs gets swapped. Don't invent new IDs.".to_string());
        // rows.push("The output objects may have to be sorted by coordinates or mass or some other property.".to_string());
        rows.push("Objects that stay stationary may be a useful landmark. A landmark may be a starting point for inserting a new object next to. Check that the examples have landmarks.\n\n".to_string());
        rows.push("Check if objects are aligned to a certain edge. Check if the mass is preserved. Count the number of object groups.".to_string());
        rows.push("Check if all the output objects agree on the same shape.".to_string());
        rows.push("Check if the shape may be a shortest path drawn between two landmarks and navigates around obstacle objects.".to_string());
        rows.push("For the objects that make it to the output, check if their shape is preserved.".to_string());
        rows.push("A line that goes from edge to edge, some condition may be satisfied above the line, but not below it.".to_string());
        // rows.push("Consider what side an object is touching another object, maybe in the output the touch points are different.".to_string());
        rows.push("Consider where two objects are touching, maybe the touch points are different in the output, such as moving from left side to the opposite side.".to_string());
        rows.push("Consider the touch point may be at the center of another object, so objects can be moved to the center point.".to_string());
        rows.push("Check how many times an object is duplicated, does it correspond to the mass of another object or some other property.".to_string());
        // rows.push("Transformations: sort, gravity towards, rotate, flipx, flipy, move, merge objects, split objects and so on.".to_string());
        // rows.push("Transformations: sort, gravity towards, rotate, flipx, flipy, move, merge objects, split objects, extract object, fit object inside another object, and so on.".to_string());
        rows.push("Transformations: sort, gravity towards, rotate, flipx, flipy, move, copy, merge objects, split objects, extract object, fit object inside another object, layout 2..5 objects in a direction while keeping the objects aligned and evenly spaced (remember to update tlbr coordinates). The transformation can be anything, it just have to be simple.".to_string());
        // rows.push("Check for splitview, sometimes the examples have a line spanning from edge to edge, dividing the grid into two halfs. Compare properties between the halfs and determine what properties are relevant.\n\n".to_string());
        // rows.push("\n\nThink step by step, what are the transformations across all the examples, that goes from the input to the output. Explain your reasoning for inserting new objects.".to_string());

        rows.push("\n\n# Task A".to_string());
        rows.push("Think step by step, what does the examples output objects have in common. Check if they are all horizontal lines. Are they sorted in a particular way.".to_string());

        rows.push("\n\n# Task B".to_string());
        rows.push("Think step by step, what are the transformations across all the examples, that goes from the input to the output. Write down your observations.".to_string());
        rows.push("Explain your reasoning for inserting new objects.".to_string());
        rows.push("Explain your reasoning for deleting existing objects.".to_string());
        // rows.push("or the predicted output the object ordering dictates the drawing order, the first output objects gets drawn first.".to_string());
        
        rows.push("\n\n# Task C".to_string());
        rows.push("Think step by step about the orientation of the output objects. Does it make sense to layout the objects in another direction. Update the object coordinates accordingly.".to_string());

        rows.push("\n\n# Task D".to_string());
        rows.push("With the following example, I want you to predict what the output should be. Print your reasoning before the prolog code.\n\n".to_string());
        rows.push("```prolog".to_string());
        for (pair_index, pair) in task.pairs.iter().enumerate() {
            let pair_index_u8: u8 = pair_index.min(255) as u8;
            let natural_language_pair_index: String = format!("{}", pair_index + 1);
            if pair.pair_type == PairType::Train {
                continue;
            }

            // Future experiment:
            // How do I loop over all the pairs. Can I do it with a single prompt, that contains all the pairs?
            // Or do I have to do prompting for each pair individually?
            // if pair_index == 4 {
            //     continue;
            // }
            // if pair_index == 5 {
            //     continue;
            // }
            
            {
                let size: ImageSize = pair.input.image.size();
                let s = format!("% Example {} input grid_width{}_height{}", natural_language_pair_index, size.width, size.height);
                rows.push(s);
            }

            {
                let object_nodeindex_vec: Vec::<NodeIndex> = task_graph.get_object_nodeindex_vec(pair_index_u8, ImageType::Input, connectivity)?;
                for object_nodeindex in &object_nodeindex_vec {
                    let s0: String = NaturalLanguageSerializer::natural_language_of_object(graph, *object_nodeindex)?;
                    let s1: String = format!("object(input{}_{}).", natural_language_pair_index, s0);
                    rows.push(s1);
                }        
            }

            rows.push("".to_string());
            {
                let grid_size: String = match task.predict_output_size_for_pair(pair) {
                    Ok(size) => {
                        format!("width{}_height{}", size.width, size.height)
                    },
                    Err(_) => {
                        format!("widthPREDICT_heightPREDICT")
                    }
                };
                let s = format!("% Example {} output grid_{}", natural_language_pair_index, grid_size);
                rows.push(s);
            }

            {
                rows.push(format!("PREDICT the text that starts with object(output{}_", natural_language_pair_index));
            }

            // Future experiment:
            // process all the test pairs. Currently it's only 1 test pair.
            break;
        }
        rows.push("```".to_string());
        // rows.push("Repeat the previous example prolog code, with PREDICT replaced with your predictions. Print reasoning first followed by the prolog code block".to_string());
        // rows.push("Repeat the previous example prolog code, with PREDICT replaced with your predictions. Leave out the input from the prolog code.".to_string());
        // rows.push("The task for you: Repeat the previous example prolog code, with PREDICT replaced with your predictions. Leave out the input from the prolog code.".to_string());
        rows.push("Repeat the previous example prolog code, with PREDICT replaced with your predictions.".to_string());

        Ok(rows.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_tlbr_positive_values() {
        // Act
        let actual: TLBR = TLBR::try_from("junk_t1_l2_b3_r4_junk").expect("ok");

        // Assert
        assert_eq!(actual.raw_top, 1);
        assert_eq!(actual.raw_left, 2);
        assert_eq!(actual.raw_bottom, 3);
        assert_eq!(actual.raw_right, 4);
    }

    #[test]
    fn test_10001_tlbr_negative_values() {
        // Act
        let actual: TLBR = TLBR::try_from("junk_t-1_l-2_b-3_r-4_junk").expect("ok");

        // Assert
        assert_eq!(actual.raw_top, -1);
        assert_eq!(actual.raw_left, -2);
        assert_eq!(actual.raw_bottom, -3);
        assert_eq!(actual.raw_right, -4);
    }

    #[test]
    fn test_10002_tlbr_swap_minmax() {
        // Act
        let actual: TLBR = TLBR::try_from("junk_t15_l20_b5_r10_junk").expect("ok");

        // Assert
        assert_eq!(actual.top, 5);
        assert_eq!(actual.left, 10);
        assert_eq!(actual.bottom, 15);
        assert_eq!(actual.right, 20);
    }

    #[test]
    fn test_20000_field_id() {
        // Act
        let actual: FieldId = FieldId::try_from("junk_idP33ffe7_junk").expect("ok");

        // Assert
        assert_eq!(actual.name, "P33ffe7");
        assert_eq!(actual.value, 3);
    }

    #[test]
    fn test_20001_field_id() {
        // Act
        let actual: FieldId = FieldId::try_from("junk_idP03hft3_junk").expect("ok");

        // Assert
        assert_eq!(actual.name, "P03hft3");
        assert_eq!(actual.value, 9);
    }

    #[test]
    fn test_30000_field_mass() {
        // Act
        let actual: FieldMass = FieldMass::try_from("junk_m42_junk").expect("ok");

        // Assert
        assert_eq!(actual.mass, 42);
    }

    #[test]
    fn test_40000_field_shape() {
        // Act
        let actual: FieldShape = FieldShape::try_from("junk_shapeUnclassified_junk").expect("ok");

        // Assert
        assert_eq!(actual.shape_name, "Unclassified");
    }

    #[test]
    fn test_40001_field_shape() {
        // Act
        let actual: FieldShape = FieldShape::try_from("junk_shapeRectangle_junk").expect("ok");

        // Assert
        assert_eq!(actual.shape_name, "Rectangle");
    }

    #[test]
    fn test_50000_field_transform() {
        // Act
        let actual: FieldTransform = FieldTransform::try_from("m16_shapeBoxWithTwoHoles_scaleUnknown, transform(rot90_rot270_flip90_flip270)).").expect("ok");

        // Assert
        assert_eq!(actual.raw, "rot90_rot270_flip90_flip270");
        let expected_transformations = HashSet::<ShapeTransformation>::from([
            ShapeTransformation::RotateCw90,
            ShapeTransformation::RotateCw270,
            ShapeTransformation::FlipXRotateCw90,
            ShapeTransformation::FlipXRotateCw270,
        ]);
        assert_eq!(actual.transformations, expected_transformations);
    }

    #[test]
    fn test_50001_field_transform() {
        // Act
        let actual: FieldTransform = FieldTransform::try_from("scalex1_scaley1, transform(rot0_rot180_flip_flip180)).").expect("ok");

        // Assert
        assert_eq!(actual.raw, "rot0_rot180_flip_flip180");
        let expected_transformations = HashSet::<ShapeTransformation>::from([
            ShapeTransformation::Normal,
            ShapeTransformation::RotateCw180,
            ShapeTransformation::FlipX,
            ShapeTransformation::FlipXRotateCw180,
        ]);
        assert_eq!(actual.transformations, expected_transformations);
    }

    #[test]
    fn test_50002_field_transform() {
        // Act
        let actual: FieldTransform = FieldTransform::try_from("scaley1, transform(all)).").expect("ok");

        // Assert
        assert_eq!(actual.raw, "all");
        assert_eq!(actual.transformations, ShapeTransformation::all());
    }

    #[test]
    fn test_60000_field_width_height() {
        // Act
        let actual: FieldWidthHeight = FieldWidthHeight::try_from("junk_width11_height22_junk").expect("ok");

        // Assert
        assert_eq!(actual.width, 11);
        assert_eq!(actual.height, 22);
    }

    #[test]
    fn test_70000_parse_ok() {
        // Arrange
        let s: String = NaturalLanguage::reply_example1();
        let s1: &str = &s;

        // Act
        let actual: NaturalLanguage = NaturalLanguage::try_from(s1).expect("ok");
        // actual.interpret();

        // Assert
        assert_eq!(actual.lines.len(), 3);
    }

    #[test]
    fn test_70100_parse_error() {
        // Arrange
        let s = "Text without code block\n\njunk\nignore";

        // Act
        let error = NaturalLanguage::try_from(s).expect_err("is supposed to fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "No code block found. Expected a code block starting with 3 backticks and prolog.");
    }

    #[test]
    fn test_70101_parse_unrecognized_stuff_inside_code_block() {
        // Arrange
        let s = r#"
```prolog
junk1.
junk2.
```
"#;

        // Act
        let error = NaturalLanguage::try_from(s).expect_err("is supposed to fail");

        // Assert
        let message = error.to_string();
        assert_eq!(message, "2 unrecognized lines inside the code block");
    }
}
