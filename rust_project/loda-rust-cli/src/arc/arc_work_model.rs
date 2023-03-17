use super::{Image, Histogram, LabelSet, PropertyInput};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Output {
    pub id: String,
    pub image: Image,
    pub test_image: Image,
    pub histogram: Histogram,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub id: String,
    pub image: Image,
    pub histogram: Histogram,
    
    /// Computed values such as: number of unique colors, width of biggest object.
    pub input_properties: HashMap<PropertyInput, u8>,

    // TODO: label_set pending to be computed
    // TODO: label_set that cannot be computed
    // TODO: rerun analyze until all pending properties have been computed
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
    pub label_set: LabelSet,
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
    pub label_set_intersection: LabelSet,
}
