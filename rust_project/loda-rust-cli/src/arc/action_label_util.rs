use super::{ActionLabel, ActionLabelSet, ImageProperty, PropertyOutput};

pub struct ActionLabelUtil;

impl ActionLabelUtil {
    pub fn is_output_size_same_as_input_size(action_label_set: &ActionLabelSet) -> bool {
        let mut same_width = false;
        let mut same_height = false;
        for label in action_label_set {
            match label {
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if *output == PropertyOutput::OutputWidth && *input == ImageProperty::Width {
                        same_width = true;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == ImageProperty::Height {
                        same_height = true;
                    }
                },
                _ => {}
            }
        }
        same_width && same_height
    }

    pub fn is_output_size_same_as_removed_rectangle_after_single_color_removal(action_label_set: &ActionLabelSet) -> bool {
        let mut same_width = false;
        let mut same_height = false;
        for label in action_label_set {
            match label {
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if *output == PropertyOutput::OutputWidth && *input == ImageProperty::WidthOfRemovedRectangleAfterSingleColorRemoval {
                        same_width = true;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == ImageProperty::HeightOfRemovedRectangleAfterSingleColorRemoval {
                        same_height = true;
                    }
                },
                _ => {}
            }
        }
        same_width && same_height
    }

    pub fn is_output_size_same_as_input_splitview(action_label_set: &ActionLabelSet) -> bool {
        if Self::is_output_size_same_as_input_splitview_x(action_label_set) {
            return true;
        }
        if Self::is_output_size_same_as_input_splitview_y(action_label_set) {
            return true;
        }

        // Future experiments:
        // Detect if the output size is the same as the input's splitview part size, but rotated.
        // Detect if there is both splits in both directions x and y.

        // if Self::experimental_is_output_size_same_as_input_splitview_rotated(action_label_set) {
        //     return true;
        // }
        false
    }

    /// The input is multiple images layouted horizontally, and the output width equals `split part size x`.
    fn is_output_size_same_as_input_splitview_x(action_label_set: &ActionLabelSet) -> bool {
        let mut same_width = false;
        let mut same_height = false;
        for label in action_label_set {
            match label {
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if *output == PropertyOutput::OutputWidth && *input == ImageProperty::SplitPartSizeX {
                        same_width = true;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == ImageProperty::Height {
                        same_height = true;
                    }
                },
                _ => {}
            }
        }
        same_width && same_height
    }

    /// The input is multiple images layouted vertically, and the output height equals `split part size y`.
    fn is_output_size_same_as_input_splitview_y(action_label_set: &ActionLabelSet) -> bool {
        let mut same_width = false;
        let mut same_height = false;
        for label in action_label_set {
            match label {
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if *output == PropertyOutput::OutputWidth && *input == ImageProperty::Width {
                        same_width = true;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == ImageProperty::SplitPartSizeY {
                        same_height = true;
                    }
                },
                _ => {}
            }
        }
        same_width && same_height
    }

    #[allow(dead_code)]
    fn experimental_is_output_size_same_as_input_splitview_rotated(action_label_set: &ActionLabelSet) -> bool {
        let mut width_is_rotated = false;
        let mut height_is_rotated = false;
        for label in action_label_set {
            match label {
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if *output == PropertyOutput::OutputWidth && *input == ImageProperty::SplitPartSizeY {
                        width_is_rotated = true;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == ImageProperty::SplitPartSizeX {
                        height_is_rotated = true;
                    }
                },
                _ => {}
            }
        }
        width_is_rotated || height_is_rotated
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_is_output_size_same_as_input_size_yes() {
        // Arrange
        let mut action_label_set = ActionLabelSet::new();
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputWidth, 
                input: ImageProperty::Width 
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: ImageProperty::Height
            };
            action_label_set.insert(label);
        }

        // Act
        let actual: bool = ActionLabelUtil::is_output_size_same_as_input_size(&action_label_set);

        // Assert
        assert_eq!(actual, true);
    }

    #[test]
    fn test_10001_is_output_size_same_as_input_size_no() {
        // Arrange
        let mut action_label_set = ActionLabelSet::new();
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputWidth, 
                input: ImageProperty::Height
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: ImageProperty::Width
            };
            action_label_set.insert(label);
        }

        // Act
        let actual: bool = ActionLabelUtil::is_output_size_same_as_input_size(&action_label_set);

        // Assert
        assert_eq!(actual, false);
    }

    #[test]
    fn test_20000_is_output_size_same_as_removed_rectangle_after_single_color_removal_yes() {
        // Arrange
        let mut action_label_set = ActionLabelSet::new();
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputWidth, 
                input: ImageProperty::WidthOfRemovedRectangleAfterSingleColorRemoval 
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: ImageProperty::HeightOfRemovedRectangleAfterSingleColorRemoval
            };
            action_label_set.insert(label);
        }

        // Act
        let actual: bool = ActionLabelUtil::is_output_size_same_as_removed_rectangle_after_single_color_removal(&action_label_set);

        // Assert
        assert_eq!(actual, true);
    }

    #[test]
    fn test_20001_is_output_size_same_as_removed_rectangle_after_single_color_removal_no() {
        // Arrange
        let mut action_label_set = ActionLabelSet::new();
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputWidth, 
                input: ImageProperty::HeightOfRemovedRectangleAfterSingleColorRemoval
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: ImageProperty::WidthOfRemovedRectangleAfterSingleColorRemoval
            };
            action_label_set.insert(label);
        }

        // Act
        let actual: bool = ActionLabelUtil::is_output_size_same_as_removed_rectangle_after_single_color_removal(&action_label_set);

        // Assert
        assert_eq!(actual, false);
    }
}
