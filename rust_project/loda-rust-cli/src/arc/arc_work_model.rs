use super::{Image, Histogram, ActionLabelSet, PropertyInput, InputLabelSet};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Output {
    pub id: String,
    pub image: Image,
    pub test_image: Image,
    pub histogram: Histogram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectType {
    NeighborAllAfterRemovalOfMostPopularIntersectionColor,
}

#[derive(Clone, Debug)]
pub struct Object {
    pub index: usize,
    pub cropped_object_image: Image,
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

#[derive(Clone, Debug)]
pub struct Pair {
    pub id: String,
    pub pair_type: PairType,
    pub input: Input,
    pub output: Output,
    pub removal_histogram: Histogram,
    pub insert_histogram: Histogram,
    pub action_label_set: ActionLabelSet,
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
}
