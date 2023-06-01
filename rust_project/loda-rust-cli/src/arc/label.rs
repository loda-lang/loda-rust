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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SingleColorObjectRectangleLabel {
    RectangleWithColor { color: u8 },
    RectangleWithSomeColor,
    SquareWithColor { color: u8 },
    SquareWithSomeColor,
    NonSquareWithColor { color: u8 },
    NonSquareWithSomeColor,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SingleColorObjectSparseLabel {
    SparseWithColor { color: u8 },
    SparseWithSomeColor,
}

/// Properties about the input image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputLabel {
    InputSymmetry { label: SymmetryLabel },
    InputGrid { label: GridLabel },
    InputSingleColorObjectRectangle { label: SingleColorObjectRectangleLabel },
    InputSingleColorObjectSparse { label: SingleColorObjectSparseLabel },

    /// Isolated noise pixels that each have `mass=1`.
    /// 
    /// A noise pixel may be connected diagonally with another noise pixel,
    /// however bigger diagonal shapes are suppressed.
    ///
    /// When all the training pairs agree on the same noise color,
    /// then that color may have some meaning.
    InputNoiseWithColor { color: u8 },

    /// Isolated noise pixels that each have `mass=1`.
    /// 
    /// A noise pixel may be connected diagonally with another noise pixel,
    /// however bigger diagonal shapes are suppressed.
    ///
    /// Each of the training pair have its own noise color,
    /// then that color may have some meaning.
    InputNoiseWithSomeColor,

    /// Both `PixelConnectivity4` and `PixelConnectivity8` yields the same child objects for a particular `color`.
    /// 
    /// When segmenting the input image into connected components, then the masks are the same
    /// for the `4 connected` pixels as the `8 connected` pixels.
    InputUnambiguousConnectivityWithColor { color: u8 },

    /// Both `PixelConnectivity4` and `PixelConnectivity8` yields the same child objects for all the colors in the input image.
    /// 
    /// When segmenting the input image into connected components, then the masks are the same
    /// for the `4 connected` pixels as the `8 connected` pixels.
    InputUnambiguousConnectivityWithAllColors,

    /// Doing flood fill along the border, and the mask of the color is still the same.
    /// 
    /// The color is touching the edges, and all pixels of this color is reachable.
    /// 
    /// There are no isolated pixels.
    InputBorderFloodFillConnectivity4AllPixelsWithColor { color: u8 },

    // Ideas for more
    // BorderMostPopularColor { color: u8 },
    // BorderLeastPopularColor { color: u8 },
    // AmbiguousEnumeratedObjects, // Does `PixelConnectivity4` and `PixelConnectivity8` yield different results
    // SplitColor { color: u8 },
    // SplitRowColor { color: u8 },
    // SplitColumnColor { color: u8 },
    // Split2Color { color: u8 },
    // Split3Color { color: u8 },
    // MostPopularObjectInteriorColorConnectivity4 { color: u8 },
    // LeastPopularObjectInteriorColorConnectivity4 { color: u8 },
    // MostPopularObjectOutlineColorConnectivity4 { color: u8 },
    // LeastPopularObjectOutlineColorConnectivity4 { color: u8 },
    // InputImageIsSingleColorObjectsMaybeWithBackgroundColor,
    // InputImageIsSingleColorObjectsWithBackgroundColor,
    // InputImageIsSingleColorObjectsWithoutBackgroundColor,
    // AllObjectsHaveTheSameSize,
    // AllSingleColorObjectsHaveTheSameSize { label: SingleColorObjectLabel },
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
    // TheOnlyOneThatIsSingleColor,
    // TheOnlyOneThatIsSingleColorAndSquare,
    // TheOnlyOneThatIsSingleColorAndNonSquare,
    // Number of holes
    // Has holes
    // Has no holes
    // BarChart
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyOutput {
    OutputWidth,
    OutputHeight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
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
    OutputSizeIsTheSameAsSingleColorObject { label: SingleColorObjectRectangleLabel },
    
    OutputImageIsSymmetricX,
    OutputImageIsSymmetricY,

    OutputImageOccurInsideInputImage { count: u16 },
    InputImageOccurInsideOutputImage { count: u16 },
    OutputImageOccurInsideInputImageOneOrMoreTimes,
    InputImageOccurInsideOutputImageOneOrMoreTimes,
    OutputImageIsPresentExactlyOnceInsideInputImage,
    InputImageIsPresentExactlyOnceInsideOutputImage,

    /// The input image is repeated 1 or more times in the output image
    /// the same number of times a particular color occur.
    /// This happens in task `cce03e0d` and task `ad7e01d0`.
    InputImageOccurInsideOutputImageSameNumberOfTimesAsColor { color: u8 },
    
    /// The input image is repeated 1 or more times in the output image
    /// the same number of times as the most popular color.
    /// This happens in task `27f8ce4f`.
    InputImageOccurInsideOutputImageSameNumberOfTimesAsTheMostPopularColorOfInputImage,

    /// The input image is repeated 1 or more times in the output image
    /// the same number of times as the least popular color.
    /// This happens in task `48f8583b`.
    InputImageOccurInsideOutputImageSameNumberOfTimesAsTheLeastPopularColorOfInputImage,

    OutputImageHistogramEqualToInputImageHistogram,
    RemovalColorIsTheMostPopularColorOfInputImage,

    OutputImageIsTheObjectWithObjectLabel { object_label: ObjectLabel },

    OutputImageIsInputImageWithChangesLimitedToPixelsWithColor { color: u8 },
    OutputImageIsInputImageWithChangesLimitedToPixelsWithMostPopularColorOfTheInputImage,
    OutputImageIsInputImageWithChangesLimitedToPixelsWithLeastPopularColorOfTheInputImage,
    
    OutputImageUniqueColorCount { count: u8 },
    OutputImageColorsComesFromInputImage,

    /// The output size is the same as the input size.
    /// Each pixel have the same number of identical pixels as in the input.
    /// Clusters of pixels are changing color between input and output.
    /// The action is usually to recolor each cluster.
    /// It does not detect if some of the clusters gets hidden.
    OutputImageHasSameStructureAsInputImage,

    OutputImageIsInputImageWithNoChangesToPixelsWithColor { color: u8 },
    InputImageIsOutputImageWithNoChangesToPixelsWithColor { color: u8 },
    
    OutputImagePreserveInputImageEdge { edge: ImageEdge },
    OutputImagePreserveInputImageCorner { corner: ImageCorner },

    // Ideas for more
    // InputImageBorderFloodFillOnlyHappensInTheNoChangeAreaWithColor { color: u8 },
    // OutputImageContainSingleColorObject { color: u8 },
    // OutputImageDoesNotContainSingleColorObject { color: u8 },
    // OutputPropertyIsEqualToNumberOfClustersWithColor { output: PropertyOutput, color: u8 },
    // OutputImageContainAllSingleColorObjectsAtTheirPosition,
    // OutputImageHasSameStructureAsInputImageWithColorPair { color0: u8, color1: u8 },
    // OutputSymmetry { label: SymmetryLabel },
    // OutputGrid { label: GridLabel },
    // OutputGridIsTheSameAsInputGrid,
    // OutputImageIsPresentInInputImageWithTwoColorWildcards, for solving "8731374e"
    // OutputImageWithSlightlyDifferentColorsIsPresentInTheInputImage,
    // OutputImagePreservesRowOfInputImageFromTop { row: u8 },
    // OutputImagePreservesRowOfInputImageFromBottom { row: u8 },
    // OutputImagePreservesColumnOfInputImageFromLeft { row: u8 },
    // OutputImagePreservesColumnOfInputImageFromRight { row: u8 },
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
    // AllOutputImagesAgreeOnTheSameColors.
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
