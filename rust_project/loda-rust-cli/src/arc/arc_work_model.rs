use super::{Image, ImageSize, Histogram, ObjectLabel, ActionLabelSet, ImageProperty, ImageLabelSet, Symmetry, Grid, GridPattern, SingleColorObject, OutputSpecification};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Output {
    pub id: String,

    /// The `image` is only available for the `train` pairs.
    /// 
    /// The `image` is not available for the `test` pairs.
    /// It's up to the solver to predict what the `image` should be for the `test` pairs.
    pub image: Image,

    /// For the public ARC dataset, the expected output image is available. But is not available for the private ARC dataset.
    /// When comparing if the prediction was correct, then it's the `test_image` that should be used.
    /// However since it's not available for the private ARC dataset.
    pub test_image: Image,

    /// The meta data that can be extracted from the `train` output image.
    /// 
    /// It's not available for the `test` pairs, and the fields are `None` or empty.
    pub image_meta: ImageMeta,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ObjectType {
    RemovalOfMostPopularColorInThisImageAfterwardSegmentByNeighborAll,

    // Ideas for more object types
    // RemovalOfMostPopularColorAcrossAllImagesAfterwardSegmentByNeighborAll,
    // SegmentByGrid,
    // SegmentByColor,
}

#[derive(Clone, Debug)]
pub struct Object {
    pub index: usize,
    pub cropped_object_image: Image,
    pub object_label_set: HashSet<ObjectLabel>,
}

/// Data about the image.
#[derive(Clone, Debug)]
pub struct ImageMeta {
    /// Histogram of all the pixels in the image.
    pub histogram_all: Histogram,

    /// Histogram of the pixels on the border of the image.
    pub histogram_border: Histogram,

    /// Computed values such as: is symmetric x, is symmetric y.
    pub image_label_set: ImageLabelSet,

    pub grid: Option<Grid>,

    pub symmetry: Option<Symmetry>,
    
    pub single_color_object: Option<SingleColorObject>,

    /// Computed values such as: number of unique colors, width of biggest object.
    pub image_properties: HashMap<ImageProperty, u8>,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub id: String,
    pub image: Image,
    pub image_meta: ImageMeta,

    /// The identified objects
    pub input_objects: HashMap<ObjectType, Vec<Object>>,

    pub repair_mask: Option<Image>,
    pub repaired_image: Option<Image>,

    pub grid_pattern: Option<GridPattern>,

    pub enumerated_objects: Option<Image>,

    pub substitution_rule_applied: Option<Image>,
    
    pub predicted_single_color_image: Option<Image>,

    pub removal_color: Option<u8>,

    pub most_popular_intersection_color: Option<u8>,

    pub single_pixel_noise_color: Option<u8>,

    // Future experiments to do.
    // least_popular_intersection_color
    //
    // Resolving these properties is similar to a package manager, a DAG (directed acyclic graph).
    // One property may depend on another property that depends on a third property.
    // As of 2023-04-25 the code is not a DAG, and gets initialized in a kludgy way. I want to migrate to a DAG.
    // State keeping of the input_properties. 
    // Computed, NotYetComputed, CannotBeComputed, DependingOnAnotherProperty.
    // Rerun analytics until all pending properties have been computed
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairType {
    Train,
    Test,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Prediction {
    OutputSize { size: ImageSize },
    OutputPalette { histogram: Histogram },
    OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color: u8 },

    // Ideas for more
    // background color, the most popular intersection color in input and output, across all training pairs
    // substitution, replace this image with that image
    // replace this color with that color
    // weak prediction: the color is a subset of this palette.
    // weak prediction: background_color
    // weak prediction: the width is a in a range.
    // weak prediction: the height is a in a range.
    // weak prediction: the size must be a square.
    // is the input grid preserved in the output
    // grid color
}

pub type PredictionSet = HashSet<Prediction>;

#[derive(Clone, Debug)]
pub struct Pair {
    /// The position in the `task.pairs` vector.
    /// 
    /// This is different than in the ARC dataset json file. 
    /// Where there are two vectors with pairs, one for the `train` pairs, and one for the `test` pairs.
    pub pair_index: u8,

    /// Shows the position in the ARC dataset json file.
    /// Where there are two vectors with pairs, one for the `train` pairs, and one for the `test` pairs.
    /// 
    /// When the pair is contained in the `train` pairs vector, then the id is `train[train_index]`.
    /// 
    /// When the pair is contained in the `test` pairs vector, then the id is `train[test_index]`.
    pub id: String,

    pub pair_type: PairType,
    pub input: Input,
    pub output: Output,
    pub removal_histogram: Histogram,
    pub insert_histogram: Histogram,
    pub action_label_set: ActionLabelSet,
    pub prediction_set: PredictionSet,
    pub output_specification_vec: Vec<OutputSpecification>,

    /// What does the `input.image_meta.image_label_set` have in common with the `output.image_meta.image_label_set`.
    pub input_output_image_label_set_intersection: ImageLabelSet,

    /// Computed image properties, by comparing input with output.
    /// 
    /// These properties are only available for the `train` pairs. 
    /// 
    /// These are not available for the `test` pairs, since you are supposed to predict the output.
    /// So looking at the output is not allowed.
    pub input_output_image_properties: HashMap<ImageProperty, u8>,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub pairs: Vec<Pair>,
    pub input_histogram_union: Histogram,
    pub input_histogram_intersection: Histogram,
    pub output_histogram_union: Histogram,
    pub output_histogram_intersection: Histogram,
    pub removal_histogram_intersection: Histogram,
    pub insert_histogram_intersection: Histogram,

    /// What do the `input` images have in common across the `train` and `test` pairs.
    pub input_properties_intersection: HashMap<ImageProperty, u8>,

    /// What do the `input` images have in common across the `train` and `test` pairs.
    pub input_image_label_set_intersection: ImageLabelSet,

    /// What do the `output` images have in common across the `train` pairs. Not available for the `test` pairs.
    pub output_image_label_set_intersection: ImageLabelSet,

    /// What do the `input` images and the `output` images have in common across the `train` pairs. Not available for the `test` pairs.
    pub input_output_image_label_set_intersection: ImageLabelSet,

    pub action_label_set_intersection: ActionLabelSet,
    pub occur_in_solutions_csv: bool,
}
