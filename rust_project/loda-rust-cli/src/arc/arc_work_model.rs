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
