use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyInput {
    InputWidth,
    InputWidthPlus1,
    InputWidthPlus2,
    InputWidthMinus1,
    InputWidthMinus2,
    InputHeight,
    InputHeightPlus1,
    InputHeightPlus2,
    InputHeightMinus1,
    InputHeightMinus2,
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
    InputWidthOfRemovedRectangleAfterSingleColorRemoval,
    InputHeightOfRemovedRectangleAfterSingleColorRemoval,

    // Ideas for more
    // InputUniqueColorCountAfterRemoval
    // Number of 1px lines horizontal
    // Number of 1px lines vertical
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputLabel {
    InputImageIsSymmetricX,
    InputImageIsSymmetricY,

    // Ideas for more
    // InputUniqueColors { color: Vec<u8> },
    // InputAspectRatio { width: u8, height: u8 },
}

pub type InputLabelSet = HashSet<InputLabel>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ObjectLabel {
    TheOnlyOneWithSmallestArea,
    TheOnlyOneWithBiggestArea,
    TheOnlyOneWithSymmetryX,
    TheOnlyOneWithAsymmetryX,
    TheOnlyOneWithSymmetryY,
    TheOnlyOneWithAsymmetryY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyOutput {
    OutputWidth,
    OutputHeight,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionLabel {
    OutputPropertyIsEqualToInputProperty { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsInputPropertyMultipliedBy { output: PropertyOutput, input: PropertyInput, scale: u8 },
    OutputPropertyIsInputPropertyMultipliedBySomeScale { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsInputPropertyMultipliedByInputSize { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsInputPropertyDividedBy { output: PropertyOutput, input: PropertyInput, scale: u8 },
    OutputPropertyIsInputPropertyDividedBySomeScale { output: PropertyOutput, input: PropertyInput },
    OutputPropertyIsConstant { output: PropertyOutput, value: u8 },

    OutputImageIsSymmetricX,
    OutputImageIsSymmetricY,

    OutputImageOccurInsideInputImage { count: u16 },
    InputImageOccurInsideOutputImage { count: u16 },
    OutputImageOccurInsideInputImageOneOrMoreTimes,
    InputImageOccurInsideOutputImageOneOrMoreTimes,
    OutputImageIsPresentExactlyOnceInsideInputImage,
    InputImageIsPresentExactlyOnceInsideOutputImage,

    OutputImageHistogramEqualToInputImageHistogram,
    RemovalColorIsThePrimaryColorOfInputImage,

    OutputImageIsTheObjectWithObjectLabel { object_label: ObjectLabel },

    OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color: u8 },
    OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage,
    OutputImageIsInputImageWithChangesLimitedToPixelsWithLeastPopularColorOfTheInputImage,
    
    OutputImageUniqueColorCount { count: u8 },
    OutputImageColorsComesFromInputImage,
    
    // Ideas for more
    // OutputImageIsSingleColor,
    // OutputMaskIsTheSameAsInputMask,
    // OutputMaskIsASubsetOfInputMask,
    // InputMaskIsASubsetOfOutputMask,
    // AllObjectsHaveTheSameSizeAsTheOutputImage
    // OutputImageRowsAllPresentInTheInputImage,
    // OutputImageColumnsAllPresentInTheInputImage,
    // OutputPropertySamePixelValuesAsInput { count_same: u16, count_different: u16 },
    // OutputSizeIsInputSizeAddConstant
    // OutputSizeIsInputSizeMultipliedByWithPadding
    // OutputAspectRatio { width: u8, height: u8 },
    // OutputAspectRatioEqualToInputAspectRatio,
}

pub type ActionLabelSet = HashSet<ActionLabel>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_10000_intersection() {
        let mut set0: ActionLabelSet = HashSet::new();
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputHeight
            };
            set0.insert(label);
        }

        let mut set1: ActionLabelSet = HashSet::new();
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: PropertyInput::InputHeight
            };
            set1.insert(label);
        }

        let set2: ActionLabelSet = set0.intersection(&set1).map(|l| l.clone()).collect();
        let expected_label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
            output: PropertyOutput::OutputHeight,
            input: PropertyInput::InputHeight
        };
        assert_eq!(set2.contains(&expected_label), true);
    }
}
