use super::{ActionLabel, ActionLabelSet, PropertyInput, PropertyOutput};

pub struct ActionLabelUtil;

impl ActionLabelUtil {
    pub fn is_output_size_same_as_input_size(action_label_set: &ActionLabelSet) -> bool {
        let mut same_width = false;
        let mut same_height = false;
        for label in action_label_set {
            match label {
                ActionLabel::OutputPropertyIsEqualToInputProperty { output, input } => {
                    if *output == PropertyOutput::OutputWidth && *input == PropertyInput::InputWidth {
                        same_width = true;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == PropertyInput::InputHeight {
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
                    if *output == PropertyOutput::OutputWidth && *input == PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval {
                        same_width = true;
                    }
                    if *output == PropertyOutput::OutputHeight && *input == PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval {
                        same_height = true;
                    }
                },
                _ => {}
            }
        }
        same_width && same_height
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
                input: PropertyInput::InputWidth 
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputHeight
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
                input: PropertyInput::InputHeight
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputWidth
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
                input: PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval 
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval
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
                input: PropertyInput::InputHeightOfRemovedRectangleAfterSingleColorRemoval
            };
            action_label_set.insert(label);
        }
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputWidthOfRemovedRectangleAfterSingleColorRemoval
            };
            action_label_set.insert(label);
        }

        // Act
        let actual: bool = ActionLabelUtil::is_output_size_same_as_removed_rectangle_after_single_color_removal(&action_label_set);

        // Assert
        assert_eq!(actual, false);
    }
}
