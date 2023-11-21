use std::collections::HashSet;

/// Properties about the `input` image, or the `output` image. These properties all have value `u8`.
/// 
/// These properties are used for reasoning about what the size of the output image may be.
/// Usually it's the width and height of the input image that is being used.
/// The values being used are in the range `[0..30]`.
/// 
/// Extreme values in the range `[31..255]`, occur frequently. These are not filtered out.
/// It's rare that extreme values are being used for computing the output size.
/// 
/// All the `ImageProperty` values can be computed for a `test pair` without accessing the output image.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageProperty {
    Width,
    WidthPlus1,
    WidthPlus2,
    WidthMinus1,
    WidthMinus2,
    Height,
    HeightPlus1,
    HeightPlus2,
    HeightMinus1,
    HeightMinus2,
    BiggestValueThatDividesWidthAndHeight,
    UniqueColorCount,
    UniqueColorCountMinus1,
    NumberOfPixelsWithMostPopularColor,
    NumberOfPixelsWith2ndMostPopularColor,
    WidthOfPrimaryObjectAfterSingleColorRemoval,
    HeightOfPrimaryObjectAfterSingleColorRemoval,
    MassOfPrimaryObjectAfterSingleColorRemoval,
    WidthOfPrimaryObjectAfterSingleIntersectionColor,
    HeightOfPrimaryObjectAfterSingleIntersectionColor,
    MassOfPrimaryObjectAfterSingleIntersectionColor,
    NumberOfPixelsCorrespondingToTheSingleIntersectionColor,
    NumberOfPixelsNotCorrespondingToTheSingleIntersectionColor,
    WidthOfRemovedRectangleAfterSingleColorRemoval,
    HeightOfRemovedRectangleAfterSingleColorRemoval,
    MassOfAllNoisePixels,
    UniqueNoiseColorCount,
    WidthAfterTrimBorderColor,
    WidthMinus2AfterTrimBorderColor,
    HeightAfterTrimBorderColor,
    HeightMinus2AfterTrimBorderColor,
    WidthOfBiggestObjectIgnoringMostPopularBorderColor,
    HeightOfBiggestObjectIgnoringMostPopularBorderColor,
    NumberOfClustersWithMostPopularIntersectionColor,
    NumberOfClustersWithLeastPopularIntersectionColor,
    CellCountX,
    CellCountY,
    SplitPartSizeX,
    SplitPartSizeY,
    SplitPartSizeXY,

    // Ideas for more
    // MassOfColor0, ... MassOfColor9,
    // MassOfPixelsWithSingleIntersectionRemovalColor,
    // UniqueColorCountIgnoringTheMostPopularIntersectionColor,
    // NoisePixelsCountOutsideAnyObjects,
    // MaxNumberOfClustersInSparseSingleColorObject,
    // MaxWidthOfClustersInSparseSingleColorObject,
    // MaxHeightOfClustersInSparseSingleColorObject,
    // MaxNoisePixelsInsideAnotherObject,
    // PrimaryObjectInteriorMass,
    // PrimaryObjectCornerCount,
    // CellCountHorizontal,
    // CellCountVertical,
    // UniqueColorCountAfterRemoval
    // Number of 1px lines horizontal
    // Number of 1px lines vertical
    // Number of corners in primary object
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

/// Does the image contain horizontal lines or vertical lines that are splitting the image into multiple parts
/// 
/// Properties about an input image or an output image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SplitLabel {
    SplitColor { color: u8 },
    SplitWithSomeColor,
    SplitDirectionX,
    SplitDirectionY,
    SplitDirectionSome,
    SplitSeparatorSizeX { size: u8 },
    SplitSeparatorSizeY { size: u8 },
    SplitSeparatorSizeXY { size: u8 },
    SplitSeparatorCountX { count: u8 },
    SplitSeparatorCountY { count: u8 },
    SplitPartSizeX { size: u8 },
    SplitPartSizeY { size: u8 },
    SplitPartSizeXY { size: u8 },
    SplitPartCountX { count: u8 },
    SplitPartCountY { count: u8 },
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

/// Properties used for both the input image and the output image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageLabel {
    Split { label: SplitLabel },
    Symmetry { label: SymmetryLabel },
    Grid { label: GridLabel },
    SingleColorObjectRectangle { label: SingleColorObjectRectangleLabel },
    SingleColorObjectSparse { label: SingleColorObjectSparseLabel },

    /// Isolated noise pixels that each have `mass=1`.
    /// 
    /// A noise pixel may be connected diagonally with another noise pixel,
    /// however bigger diagonal shapes are suppressed.
    ///
    /// When all the images agree on the same noise color,
    /// then that color may have some meaning.
    NoiseWithColor { color: u8 },

    /// Isolated noise pixels that each have `mass=1`.
    /// 
    /// A noise pixel may be connected diagonally with another noise pixel,
    /// however bigger diagonal shapes are suppressed.
    ///
    /// Each of the images have its own noise color,
    /// then that color may have some meaning.
    NoiseWithSomeColor,

    /// Both `PixelConnectivity4` and `PixelConnectivity8` yields the same child objects for a particular `color`.
    /// 
    /// When segmenting the image into connected components, then the masks are the same
    /// for the `4 connected` pixels as the `8 connected` pixels.
    UnambiguousConnectivityWithColor { color: u8 },

    /// Both `PixelConnectivity4` and `PixelConnectivity8` yields the same child objects for all the colors in the input image.
    /// 
    /// When segmenting the image into connected components, then the masks are the same
    /// for the `4 connected` pixels as the `8 connected` pixels.
    UnambiguousConnectivityWithAllColors,

    /// Doing flood fill along the border, and the mask of the color is still the same.
    /// 
    /// The color is touching the edges, and all pixels of this color is reachable.
    /// 
    /// There are no isolated pixels.
    BorderFloodFillConnectivity4AllPixelsWithColor { color: u8 },

    /// Only one color is used along all the borders in the image.
    SingleBorderColor { color: u8 },

    /// Every edge (top, bottom, left and right) are using the most popular border color.
    /// 
    /// There has to be at least 1 pixel on each edge.
    /// 
    /// The color is the most popular in the border histogram.
    /// 
    /// This may be the a sparse object that is touching all the edges.
    /// 
    /// If the input image only use 1 color for all pixels, then that object is touching all the edges.
    MostPopularBorderColorIsPresentOnAllEdges,

    /// If there is a pattern. This is the `horizontal length` of the pattern before it repeats.
    PeriodicityX { period: u8 },

    /// If there is a pattern. This is the `vertical length` of the pattern before it repeats.
    PeriodicityY { period: u8 },

    // Ideas for more
    // DiagonalLinesWithColor { color: u8 },
    // How many times are the pixels stretched out horizontally. Dividing by this scale does not cause dataloss.
    // ScaleX { scale: u8 },
    // How many times are the pixels stretched out vertically. Dividing by this scale does not cause dataloss.
    // ScaleY { scale: u8 },
    // TwoPixelsPositionedOnTheSameLine { direction: horz, vert, diagA, diagB }
    // AllObjectsAreMovedByTheSameOffsetNoWrap { offset_x: i32, offset_y: i32, background_color: u8 },
    // AllObjectsAreMovedByTheSameOffsetWrapAround { offset_x: i32, offset_y: i32, background_color: u8 },
    // AllObjectsFromTheInputImagePresentExactlyOnceInTheOutputImageButWithDifferentOffsets,
    // SecondMostPopularColor { color: u8 },
    // SecondLeastPopularColor { color: u8 },
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
    // ImageIsSingleColorObjectsMaybeWithBackgroundColor,
    // ImageIsSingleColorObjectsWithBackgroundColor,
    // ImageIsSingleColorObjectsWithoutBackgroundColor,
    // AllObjectsHaveTheSameSize,
    // AllSingleColorObjectsHaveTheSameSize { label: SingleColorObjectLabel },
    // ColorThatDoesNotOccurInTheIntersection { color: u8 },
    // UniqueColors { color: Vec<u8> },
    // AspectRatio { width: u8, height: u8 },
    // ContainsOneOrMoreBoxes,
    // ContainsASnakeLikePattern,
}

pub type ImageLabelSet = HashSet<ImageLabel>;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChangeItem {
    SingleLineRow, // there is a change somewhere inside the row when encountering a Red pixel.
    SingleLineColumn, // there is a change somewhere inside the column when encountering a Red pixel.
    SingleLineRowOrColumn, // there is a change somewhere inside the row or column when encountering a Red pixel.
    SingleLineDiagonalA, // there is a change somewhere inside the DiagonalA when encountering a Red pixel.
    SingleLineDiagonalB, // there is a change somewhere inside the DiagonalB when encountering a Red pixel.
    SingleLineSomeDiagonal, // there is a change somewhere inside the DiagonalA or DiagonalB when encountering a Red pixel.
    SingleLineAny45DegreeAngle, // there is a change somewhere inside the row/column/diagonals when encountering a Red pixel.

    // Future experiments
    // SingleLineRowLeft, does the row change only on the left side of the Red pixel.
    // SingleLineRowRight, does the row change only on the right side of the Red pixel.
    // SingleLineColumnTop, does the column change only on the above the Red pixel.
    // SingleLineColumnBottom, does the column change only on the below the Red pixel.
    // ThreeLineRow, there is a change somewhere inside the 3 rows when encountering a Red pixel.
    // ThreeLineColumn, there is a change somewhere inside the 3 columns when encountering a Red pixel.
    // PyramidUp, there is a change somewhere inside the a pyramid cone when encountering a Red pixel.
    // PyramidDown, there is a change somewhere inside the a pyramid cone when encountering a Red pixel.
    // PyramidLeft, there is a change somewhere inside the a pyramid cone when encountering a Red pixel.
    // PyramidRight, there is a change somewhere inside the a pyramid cone when encountering a Red pixel.
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActionLabel {
    OutputPropertyIsEqualToInputProperty { output: PropertyOutput, input: ImageProperty },
    OutputPropertyIsInputPropertyMultipliedBy { output: PropertyOutput, input: ImageProperty, scale: u8 },
    OutputPropertyIsInputPropertyMultipliedBySomeScale { output: PropertyOutput, input: ImageProperty },
    OutputPropertyIsInputPropertyMultipliedByInputSize { output: PropertyOutput, input: ImageProperty },
    OutputPropertyIsInputPropertyDividedBy { output: PropertyOutput, input: ImageProperty, scale: u8 },
    OutputPropertyIsInputPropertyDividedBySomeScale { output: PropertyOutput, input: ImageProperty },
    OutputPropertyIsInputPropertySquared { output: PropertyOutput, input: ImageProperty },
    OutputPropertyIsConstant { output: PropertyOutput, value: u8 },
    OutputSizeIsTheSameAsSingleColorObject { label: SingleColorObjectRectangleLabel },
    OutputSizeIsTheSameAsBoundingBoxOfColor { color: u8 },
    OutputSizeIsTheSameAsRotatedBoundingBoxOfColor { color: u8 },
    
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

    /// What input colors explains the output colors. The most popular stays the most popular.
    InputMostPopularColorIsOutputMostPopularColor,

    /// What input colors explains the output colors. The least popular stays the least popular.
    InputLeastPopularColorIsOutputLeastPopularColor,

    /// What input colors explains the output colors. The most popular becomes the least popular.
    InputMostPopularColorIsOutputLeastPopularColor,

    /// What input colors explains the output colors. The least popular becomes the most popular.
    InputLeastPopularColorIsOutputMostPopularColor,

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

    /// The input histogram is the same colors and counters as the output histogram, in a particular direction.
    HistogramSameColorsAndSameCountsForColumns,
    HistogramSameColorsAndSameCountsForRows,

    /// The input histogram have the same colors (may have different counters) as the output histogram, in a particular direction.
    HistogramSameColorsIgnoringCountsForColumns,
    HistogramSameColorsIgnoringCountsForRows,

    /// The input histogram have the same counters (may have different colors) as the output histogram, in a particular direction.
    HistogramSameCountsIgnoringColorsForColumns,
    HistogramSameCountsIgnoringColorsForRows,

    /// Preserves content, but the ordering of the columns are changed.
    ChangesOrderOfColumns,

    /// Preserves content, but the ordering of the rows are changed.
    ChangesOrderOfRows,
    
    /// Preserves content, but the ordering of the columns or rows are changed.
    ChangesOrderOfColumnsOrRows,

    /// The output is not an exact copy of the input, whenever a particular color occur.
    /// Example: If Red is present in the input row, then the output row gets some kind of modification.
    ChangeHappensToItemWithColor { item: ChangeItem, color: u8 },

    /// The output is an exact copy of the input, whenever a particular color occur.
    /// Example: If Red is present in the input row, then the output row is a copy of the input.
    NoChangeHappensToItemWithColor { item: ChangeItem, color: u8 },

    // Ideas for more
    // ChangeHappensToItemWithSpecialColor { item: ChangeItem, color: SpecialColor }, // where SpecialColor is most popular, least popular, noise color, grid color.
    // NoChangeHappensToItemWithSpecialColor { item: ChangeItem, color: SpecialColor }, // where SpecialColor is most popular, least popular, noise color, grid color.
    // PeriodicityXIgnoringBackgroundColor { period: u8 },
    // PeriodicityYIgnoringBackgroundColor { period: u8 },
    // HistogramNoChangeInDirectionDiagonalA,
    // HistogramNoChangeInDirectionDiagonalB,
    // Directionality of a pair. Is it up/down/left/right. Then the images will require the corresponding rotation applied and unapplied.
    // OutputSizeIsTheSameAsSplitViewCell,
    // Preserve image above the split view, and change the image below the separator
    // InputStatsIsOutputStats { mode: ImageStatsMode },
    // InputStatsHasHigherMeanAndLowerSigmaThanOutputStats { mode: ImageStatsMode },
    // InputStatsHasLowerMeanAndHigherSigmaThanOutputStats { mode: ImageStatsMode },
    // InputStatsHasNonZeroSigmaAndOutputStatsHasSigmaZero { mode: ImageStatsMode },
    // OutputSizeIsTheSameAsObjectAtPosition { position: Top/Bottom/Left/Right },
    // NoMovementInDirectionX,
    // NoMovementInDirectionY,
    // ObjectsOnlyMoveInDirectionX,
    // ObjectsOnlyMoveInDirectionY,
    // CropWhenFinishedDrawingInsideSingleColorObjectWithColor { color: u8 },
    // OutputImageCropOutSingleColorObject { color: u8 },
    // OutputSizeIsTheSameAsBoundingBoxOfSingleColorObject { color: u8 },
    // OutputSizeIsTheSameAsHoleInSingleColorObject { color: u8 },
    // OutputSizeIsTheSameAsInputSmallestSingleColorObjectRectangle,
    // OutputSizeIsTheSameAsInputSmallestSingleColorObjectSquare,
    // OutputSizeIsTheSameAsInputSmallestSingleColorObjectNonSquare,
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
                input: ImageProperty::Height
            };
            set0.insert(label);
        }

        let mut set1: ActionLabelSet = HashSet::new();
        {
            let label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
                output: PropertyOutput::OutputHeight,
                input: ImageProperty::Height
            };
            set1.insert(label);
        }

        let set2: ActionLabelSet = set0.intersection(&set1).map(|l| l.clone()).collect();
        let expected_label = ActionLabel::OutputPropertyIsEqualToInputProperty { 
            output: PropertyOutput::OutputHeight,
            input: ImageProperty::Height
        };
        assert_eq!(set2.contains(&expected_label), true);
    }
}
