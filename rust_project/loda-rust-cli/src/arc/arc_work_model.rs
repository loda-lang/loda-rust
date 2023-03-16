use super::{Image, Histogram};
use super::{Label, LabelSet, PropertyInput, PropertyOutput};
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
pub enum BufferPairType {
    Train,
    Test,
}

#[derive(Clone, Debug)]
pub struct Pair {
    pub id: String,
    pub pair_type: BufferPairType,
    pub input: Input,
    pub output: Output,
    pub removal_histogram: Histogram,
    pub insert_histogram: Histogram,
    pub label_set: LabelSet,
}
