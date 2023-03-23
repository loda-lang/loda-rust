use super::{Image, ImageSize, Histogram, ObjectLabel, ActionLabelSet, PropertyInput, InputLabelSet};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Output {
    pub id: String,
    pub image: Image,
    pub test_image: Image,
    pub histogram: Histogram,
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

#[derive(Clone, Debug)]
pub struct Input {
    pub id: String,
    pub image: Image,
    pub histogram: Histogram,
    
    /// Computed values such as: number of unique colors, width of biggest object.
    pub input_properties: HashMap<PropertyInput, u8>,

    /// Computed values such as: is symmetric x, is symmetric y.
    pub input_label_set: InputLabelSet,

    /// The identified objects
    pub input_objects: HashMap<ObjectType, Vec<Object>>,

    // Future experiments to do.
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

    // Ideas for more
    // substitution, replace this image with that image
    // replace this color with that color
    // weak prediction: the color is a subset of this palette.
    // weak prediction: background_color
    // weak prediction: the width is a in a range.
    // weak prediction: the height is a in a range.
    // weak prediction: the size must be a square.
}

pub type PredictionSet = HashSet<Prediction>;

#[derive(Clone, Debug)]
pub struct Pair {
    pub id: String,
    pub pair_type: PairType,
    pub input: Input,
    pub output: Output,
    pub removal_histogram: Histogram,
    pub insert_histogram: Histogram,
    pub action_label_set: ActionLabelSet,
    pub prediction_set: PredictionSet,
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
    pub input_properties_intersection: HashMap<PropertyInput, u8>,
    pub input_label_set_intersection: InputLabelSet,
    pub action_label_set_intersection: ActionLabelSet,
    pub occur_in_solutions_csv: bool,
}
