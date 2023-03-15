use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyInput {
    InputWidth,
    InputWidthPlus1,
    InputWidthPlus2,
    InputHeight,
    InputHeightPlus1,
    InputHeightPlus2,
    InputUniqueColorCount,
    InputUniqueColorCountMinus1,
    InputNumberOfPixelsWithMostPopularColor,
    InputNumberOfPixelsWith2ndMostPopularColor,
    InputWidthOfPrimaryObjectAfterSingleColorRemoval,
    InputHeightOfPrimaryObjectAfterSingleColorRemoval,
    InputMassOfPrimaryObjectAfterSingleColorRemoval,
    InputWidthOfPrimaryObjectAfterSingleIntersectionColor,
    InputHeightOfPrimaryObjectAfterSingleIntersectionColor,
    InputMassOfPrimaryObjectAfterSingleIntersectionColor,
    InputNumberOfPixelsCorrespondingToTheSingleIntersectionColor,
    InputNumberOfPixelsNotCorrespondingToTheSingleIntersectionColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyOutput {
    OutputWidth,
    OutputHeight,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Label {
    OutputSizeWidth { width: u8 },
    OutputSizeHeight { height: u8 },
    OutputPropertyIsEqualToInputProperty { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsInputPropertyMultipliedBy { output: PropertyOutput, input: PropertyInput, scale: u8 },
    OutputPropertyIsInputPropertyMultipliedBySomeScale { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsInputPropertyMultipliedByInputSize { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsInputPropertyDividedBy { output: PropertyOutput, input: PropertyInput, scale: u8 },
    OutputPropertyIsInputPropertyDividedBySomeScale { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsConstant { output: PropertyOutput, value: u8, reason: String },

    // Ideas for more
    // OutputImageIsPresentInInputImage
    // OutputPropertySamePixelValuesAsInput { count_same: u16, count_different: u16 },
    // OutputSizeIsInputSizeAddConstant
    // OutputSizeIsInputSizeMultipliedByWithPadding
    // InputUniqueColors { color: Vec<u8> },
    // InputAspectRatio { width: u8, height: u8 },
    // OutputAspectRatio { width: u8, height: u8 },
    // OutputAspectRatioEqualToInputAspectRatio,
}

pub type LabelSet = HashSet<Label>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_intersection() {
        let mut set0: LabelSet = HashSet::new();
        {
            let label = Label::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputHeight
            };
            set0.insert(label);
        }

        let mut set1: LabelSet = HashSet::new();
        {
            let label = Label::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputHeight
            };
            set1.insert(label);
        }

        let set2: LabelSet = set0.intersection(&set1).map(|l| l.clone()).collect();
        let expected_label = Label::OutputPropertyIsEqualToInputProperty { 
            output: PropertyOutput::OutputHeight,
            input: PropertyInput::InputHeight
        };
        assert_eq!(set2.contains(&expected_label), true);
    }
}
