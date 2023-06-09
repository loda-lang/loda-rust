use super::{SingleColorObjects, ImageLabel, ImageLabelSet, SingleColorObjectRectangleLabel, SingleColorObjectSparseLabel};

pub trait SingleColorObjectToLabel {
    fn to_image_label_set(&self) -> anyhow::Result<ImageLabelSet>;
}

impl SingleColorObjectToLabel for SingleColorObjects {
    fn to_image_label_set(&self) -> anyhow::Result<ImageLabelSet> {
        let mut image_label_set = ImageLabelSet::new();
        for object in &self.rectangle_vec {
            {
                let label = SingleColorObjectRectangleLabel::RectangleWithColor { color: object.color };
                let image_label = ImageLabel::SingleColorObjectRectangle { label };
                image_label_set.insert(image_label);
            }
            {
                let label = SingleColorObjectRectangleLabel::RectangleWithSomeColor;
                let image_label = ImageLabel::SingleColorObjectRectangle { label };
                image_label_set.insert(image_label);
            }
            if object.is_square {
                {
                    let label = SingleColorObjectRectangleLabel::SquareWithColor { color: object.color };
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    image_label_set.insert(image_label);
                }
                {
                    let label = SingleColorObjectRectangleLabel::SquareWithSomeColor;
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    image_label_set.insert(image_label);
                }
            } else {
                {
                    let label = SingleColorObjectRectangleLabel::NonSquareWithColor { color: object.color };
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    image_label_set.insert(image_label);
                }
                {
                    let label = SingleColorObjectRectangleLabel::NonSquareWithSomeColor;
                    let image_label = ImageLabel::SingleColorObjectRectangle { label };
                    image_label_set.insert(image_label);
                }
            }
        }
        for object in &self.sparse_vec {
            {
                let label = SingleColorObjectSparseLabel::SparseWithColor { color: object.color };
                let image_label = ImageLabel::SingleColorObjectSparse { label };
                image_label_set.insert(image_label);
            }
            {
                let label = SingleColorObjectSparseLabel::SparseWithSomeColor;
                let image_label = ImageLabel::SingleColorObjectSparse { label };
                image_label_set.insert(image_label);
            }
        }
        {
            for object in &self.rectangle_vec {
                let image_label = ImageLabel::UnambiguousConnectivityWithColor { color: object.color };
                image_label_set.insert(image_label);
            }
            let mut all_are_connectivity48_identical = true;
            for object in &self.sparse_vec {
                if !object.connectivity48_identical {
                    all_are_connectivity48_identical = false;
                    continue;
                }
                let image_label = ImageLabel::UnambiguousConnectivityWithColor { color: object.color };
                image_label_set.insert(image_label);
            }
            if all_are_connectivity48_identical {
                let image_label = ImageLabel::UnambiguousConnectivityWithAllColors;
                image_label_set.insert(image_label);
            }
        }
        if let Some(color) = self.single_pixel_noise_color() {
            {
                let image_label = ImageLabel::NoiseWithColor { color };
                image_label_set.insert(image_label);
            }
            {
                let image_label = ImageLabel::NoiseWithSomeColor;
                image_label_set.insert(image_label);
            }
        }
        Ok(image_label_set)
    }
}
