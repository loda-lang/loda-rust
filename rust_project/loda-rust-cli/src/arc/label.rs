use std::collections::HashSet;

/// Properties about the input image. These properties all have value `u8`.
/// 
/// These properties are used for reasoning about what the size of the output image may be.
/// Usually it's the width and height of the input image that is being used.
/// The values being used are in the range `[0..30]`.
/// 
/// Extreme values in the range `[31..255]`, occur frequently. These are not filtered out.
/// It's rare that extreme values are being used for computing the output size.
/// 
/// All the `PropertyInput` values can be computed for a `test pair` without accessing the output image.
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
    InputBiggestValueThatDividesWidthAndHeight,
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
    // InputCellCountHorizontal,
    // InputCellCountVertical,
    // InputUniqueColorCountAfterRemoval
    // Number of 1px lines horizontal
    // Number of 1px lines vertical
}

/// Does the image contain symmetric patterns
/// 
/// Properties about an input image or an output image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SymmetryLabel {
    Horizontal,
    HorizontalWithMismatches,
    HorizontalWithInset,
    HorizontalWithInsetAndMismatches,
    Vertical,
    VerticalWithMismatches,
    VerticalWithInset,
    VerticalWithInsetAndMismatches,
    DiagonalA,
    DiagonalAWithMismatches,
    DiagonalB,
    DiagonalBWithMismatches,

    // Ideas for more
    // RepairColor { color: u8 },
    // Number of palindromic rows { count: u8 },
    // Number of palindromic columns { count: u8 },
}

/// Does the image contain grid patterns
/// 
/// Properties about an input image or an output image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GridLabel {
    GridColor { color: u8 },
    GridWithSomeColor,
    GridWithMismatchesAndColor { color: u8 },
    GridWithMismatchesAndSomeColor,

    // Ideas for more
    // Horizontal Line color
    // Vertical Line color
    // AnyDirection Line color
    // Color only occur in the grid lines
    // Periodicity,
    // Cell size,
    // Line size,
    // Number of cells horizontal,
    // Number of cells vertical,
    // Number of lines horizontal,
    // Number of lines vertical,
    // NoiseColor { color: u8 },
}

/// Properties about the input image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputLabel {
    InputSymmetry { label: SymmetryLabel },
    InputGrid { label: GridLabel },

    // Ideas for more
    // AllObjectsHaveTheSameSize,
    // InputColorThatDoesNotOccurInTheIntersection { color: u8 },
    // InputUniqueColors { color: Vec<u8> },
    // InputAspectRatio { width: u8, height: u8 },
    // InputContainsOneOrMoreBoxes,
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

    // Ideas for more
    // Number of holes
    // Has holes
    // Has no holes
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
    OutputPropertyIsInputPropertySquared { output: PropertyOutput, input: PropertyInput },
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

    OutputImageHasSameStructureAsInputImage,
    
    // Ideas for more
    // OutputImageHasSameStructureAsInputImageWithColorPair { color0: u8, color1: u8 },
    // OutputImageIsTheSameAsInputImageWithColor { color: u8 },
    // OutputSymmetry { label: SymmetryLabel },
    // OutputGrid { label: GridLabel },
    // OutputGridIsTheSameAsInputGrid,
    // OutputImageIsPresentInInputImageWithTwoColorWildcards, for solving "8731374e"
    // OutputImageWithSlightlyDifferentColorsIsPresentInTheInputImage,
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
    // OutputSizeIsBiggerThanInputSize
    // OutputSizeIsSmallerThanInputSize
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
