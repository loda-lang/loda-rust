use super::{Image, ImageToNumber, NumberToImage, ImageOffset, ImageTrim, ImageRemoveDuplicates, ImageRotate, ImageExtractRowColumn, PopularObjects};
use super::{ImageHistogram, ImageReplaceColor, ImageSymmetry, ImagePadding, ImageResize, ImageStack};
use super::{Histogram, ImageOverlay, ImageOutline, ImageDenoise, ImageNoiseColor, ImageDetectHole, ImageSetPixelWhere};
use super::{ImageRemoveGrid, ImageCreatePalette, ImageMask, ImageUnicodeFormatting, ImageNeighbour, ImageNeighbourDirection};
use loda_rust_core::unofficial_function::{UnofficialFunction, UnofficialFunctionId, UnofficialFunctionRegistry};
use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::{Signed, ToPrimitive};
use std::sync::Arc;
use anyhow::Context;

struct ImageDebugFunction {
    id: u32,
}

impl ImageDebugFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageDebugFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 0 }
    }

    fn name(&self) -> String {
        "Debug an image by printing it to console/stdout".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let input_image: Image = input0_uint.to_image()?;
        println!("image: {}", input_image.to_unicode_string());

        // no output
        Ok(vec!())
    }
}

enum ImageOffsetFunctionMode {
    Wrap,
    Clamp,
}

struct ImageOffsetFunction {
    id: u32,
    mode: ImageOffsetFunctionMode,
}

impl ImageOffsetFunction {
    fn new(id: u32, mode: ImageOffsetFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageOffsetFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImageOffsetFunctionMode::Wrap => {
                return "Adjust image offset(dx, dy) with wrap".to_string()
            },
            ImageOffsetFunctionMode::Clamp => {
                return "Adjust image offset(dx, dy) with clamp".to_string()
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let input_image: Image = input0_uint.to_image()?;

        // input1 is dx
        let dx: i32 = input[1].to_i32().context("to_i32 dx")?;

        // input2 is dy
        let dy: i32 = input[2].to_i32().context("to_i32 dy")?;

        let output_image: Image;
        match self.mode {
            ImageOffsetFunctionMode::Wrap => {
                output_image = input_image.offset_wrap(dx, dy)?;
            },
            ImageOffsetFunctionMode::Clamp => {
                output_image = input_image.offset_clamp(dx, dy)?;
            },
        }
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageRotateFunction {
    id: u32,
}

impl ImageRotateFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageRotateFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: Rotate by x * 90 degrees".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let input_image: Image = input0_uint.to_image()?;

        // input1 is x
        let x: i8 = input[1].to_i8().context("to_i8 x")?;

        let output_image: Image = input_image.rotate(x)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageTrimFunction {
    id: u32,
}

impl ImageTrimFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageTrimFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: Trim border using histogram of border pixels".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let input_image: Image = input0_uint.to_image()?;

        let output_image: Image = input_image.trim()?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

enum ImageRemoveDuplicatesFunctionMode {
    RowsAndColumns,
    Rows,
    Columns,
}

struct ImageRemoveDuplicatesFunction {
    id: u32,
    mode: ImageRemoveDuplicatesFunctionMode,
}

impl ImageRemoveDuplicatesFunction {
    fn new(id: u32, mode: ImageRemoveDuplicatesFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageRemoveDuplicatesFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImageRemoveDuplicatesFunctionMode::RowsAndColumns => {
                return "Image: Remove duplicate rows/columns".to_string();
            },
            ImageRemoveDuplicatesFunctionMode::Rows => {
                return "Image: Remove duplicate rows".to_string();
            },
            ImageRemoveDuplicatesFunctionMode::Columns => {
                return "Image: Remove duplicate columns".to_string();
            }
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let input_image: Image = input0_uint.to_image()?;

        let output_image: Image; 
        match self.mode {
            ImageRemoveDuplicatesFunctionMode::RowsAndColumns => {
                output_image = input_image.remove_duplicates()?;
            },
            ImageRemoveDuplicatesFunctionMode::Rows => {
                output_image = input_image.remove_duplicate_rows()?;
            },
            ImageRemoveDuplicatesFunctionMode::Columns => {
                output_image = input_image.remove_duplicate_columns()?;
            }
        }
        
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}


enum ImageReplaceColorFunctionMode {
    ReplaceColor,
    ReplaceColorsOtherThan,
}

struct ImageReplaceColorFunction {
    id: u32,
    mode: ImageReplaceColorFunctionMode,
}

impl ImageReplaceColorFunction {
    fn new(id: u32, mode: ImageReplaceColorFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageReplaceColorFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImageReplaceColorFunctionMode::ReplaceColor => {
                return "Image: replace color x with color y".to_string();
            },
            ImageReplaceColorFunctionMode::ReplaceColorsOtherThan => {
                return "Image: replace colors other than x with color y".to_string();
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let input_image: Image = input0_uint.to_image()?;

        // input1 is color x
        let color_x: u8 = input[1].to_u8().context("u8 from_color")?;

        // input2 is color y
        let color_y: u8 = input[2].to_u8().context("u8 to_color")?;

        let output_image: Image;
        match self.mode {
            ImageReplaceColorFunctionMode::ReplaceColor => {
                output_image = input_image.replace_color(color_x, color_y)?;
            },
            ImageReplaceColorFunctionMode::ReplaceColorsOtherThan => {
                output_image = input_image.replace_colors_other_than(color_x, color_y)?;
            },
        }

        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageReplaceColorsWithPaletteImageFunction {
    id: u32,
}

impl ImageReplaceColorsWithPaletteImageFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageReplaceColorsWithPaletteImageFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: replace colors with palette image".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let source_image: Image = input0_uint.to_image()?;

        // input1 is image
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let palette_image: Image = input1_uint.to_image()?;

        let output_image: Image = source_image.replace_colors_with_palette_image(&palette_image)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageWithColorFunction {
    id: u32,
}

impl ImageWithColorFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageWithColorFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        "Create new image with size (x, y) and filled with color z".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is width
        let image_width: u8 = input[0].to_u8().context("u8 image_width")?;

        // input1 is height
        let image_height: u8 = input[1].to_u8().context("u8 image_height")?;

        // input2 is color
        let fill_color: u8 = input[2].to_u8().context("u8 fill_color")?;

        let output_image: Image = Image::color(image_width, image_height, fill_color);
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageSetPixelFunction {
    id: u32,
}

impl ImageSetPixelFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageSetPixelFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 4, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: set pixel at (x, y) with color z".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 4 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let mut image: Image = input0_uint.to_image()?;

        // input1 is position_x
        let position_x: u8 = input[1].to_u8().context("u8 position_x")?;

        // input2 is position_y
        let position_y: u8 = input[2].to_u8().context("u8 position_y")?;

        // input3 is pixel_color 
        let pixel_color: u8 = input[3].to_u8().context("u8 pixel_color")?;

        image.set(
            position_x as i32, 
            position_y as i32, 
            pixel_color
        ).context("set pixel")?;
        let output_uint: BigUint = image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageGetPixelFunction {
    id: u32,
}

impl ImageGetPixelFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageGetPixelFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: get pixel at (x, y)".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is position_x
        let position_x: u8 = input[1].to_u8().context("u8 position_x")?;

        // input2 is position_y
        let position_y: u8 = input[2].to_u8().context("u8 position_y")?;

        let pixel_color: u8 = image.get(
            position_x as i32, 
            position_y as i32, 
        ).context("get pixel")?;
        let output: BigInt = pixel_color.to_bigint().context("u8 to BigInt")?;
        Ok(vec![output])
    }
}

enum ImageGetAttributeFunctionMode {
    Width,
    Height,
}

struct ImageGetAttributeFunction {
    id: u32,
    mode: ImageGetAttributeFunctionMode,
}

impl ImageGetAttributeFunction {
    fn new(id: u32, mode: ImageGetAttributeFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageGetAttributeFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImageGetAttributeFunctionMode::Width => {
                return "Get width of image".to_string();
            },
            ImageGetAttributeFunctionMode::Height => {
                return "Get height of image".to_string();
            }
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        let value: BigInt;
        match self.mode {
            ImageGetAttributeFunctionMode::Width => {
                value = image.width().to_bigint().context("u8 to BigInt")?;
            },
            ImageGetAttributeFunctionMode::Height => {
                value = image.height().to_bigint().context("u8 to BigInt")?;
            }
        }
        Ok(vec![value])
    }
}

enum ImageFlipFunctionMode {
    FlipX,
    FlipY,
    FlipXY,
}

struct ImageFlipFunction {
    id: u32,
    mode: ImageFlipFunctionMode,
}

impl ImageFlipFunction {
    fn new(id: u32, mode: ImageFlipFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageFlipFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        let s: &str = match self.mode {
            ImageFlipFunctionMode::FlipX => "Image: flip x",
            ImageFlipFunctionMode::FlipY => "Image: flip y",
            ImageFlipFunctionMode::FlipXY => "Image: flip x and y",
        };
        s.to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let mut image: Image = input0_uint.to_image()?;

        match self.mode {
            ImageFlipFunctionMode::FlipX => {
                image = image.flip_x()?;
            },
            ImageFlipFunctionMode::FlipY => {
                image = image.flip_y()?;
            },
            ImageFlipFunctionMode::FlipXY => {
                image = image.flip_xy()?;
            },
        }
        let output_uint: BigUint = image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

#[derive(Debug)]
enum ImagePaddingFunctionMode {
    Top,
    Bottom,
    Left,
    Right,
}

struct ImagePaddingFunction {
    id: u32,
    mode: ImagePaddingFunctionMode,
}

impl ImagePaddingFunction {
    fn new(id: u32, mode: ImagePaddingFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImagePaddingFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImagePaddingFunctionMode::Top => {
                return "top padding by N rows with color".to_string();
            },
            ImagePaddingFunctionMode::Bottom => {
                return "bottom padding by N rows with color".to_string();
            },
            ImagePaddingFunctionMode::Left => {
                return "left padding by N columns with color".to_string();
            },
            ImagePaddingFunctionMode::Right => {
                return "right padding by N columns with color".to_string();
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let mut image: Image = input0_uint.to_image()?;

        // input1 is number of rows/columns
        let n: u8 = input[1].to_u8().context("u8 padding_count")?;

        // input2 is pixel_color 
        let pixel_color: u8 = input[2].to_u8().context("u8 pixel_color")?;

        match self.mode {
            ImagePaddingFunctionMode::Top => {
                image = image.padding_advanced(n, 0, 0, 0, pixel_color)?;
            },
            ImagePaddingFunctionMode::Bottom => {
                image = image.padding_advanced(0, 0, 0, n, pixel_color)?;
            },
            ImagePaddingFunctionMode::Left => {
                image = image.padding_advanced(0, n, 0, 0, pixel_color)?;
            },
            ImagePaddingFunctionMode::Right => {
                image = image.padding_advanced(0, 0, n, 0, pixel_color)?;
            },
        }
        let output_uint: BigUint = image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageResizeFunction {
    id: u32,
}

impl ImageResizeFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageResizeFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        format!("Resize image to size width x height")
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is width
        let width: u8 = input[1].to_u8().context("u8 width")?;

        // input2 is height
        let height: u8 = input[2].to_u8().context("u8 height")?;

        let output_image = image.resize(width, height)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageStackFunction {
    id: u32,
    inputs: u8,
    is_hstack: bool
}

impl ImageStackFunction {
    fn hstack(id: u32, inputs: u8) -> Self {
        Self {
            id,
            inputs,
            is_hstack: true,
        }
    }

    fn vstack(id: u32, inputs: u8) -> Self {
        Self {
            id,
            inputs,
            is_hstack: false,
        }
    }
}

impl UnofficialFunction for ImageStackFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: self.inputs, outputs: 1 }
    }

    fn name(&self) -> String {
        if self.is_hstack {
            return format!("Image.hstack. horizontal stack of {} images", self.inputs);
        } else {
            return format!("Image.vstack. vertical stack of {} images", self.inputs);
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != (self.inputs as usize) {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // all inputs are images
        let mut images = Vec::<Image>::with_capacity(input.len());
        for (index, input_item) in input.iter().enumerate() {
            if input_item.is_negative() {
                return Err(anyhow::anyhow!("Input[{}] must be non-negative", index));
            }
            let input_uint: BigUint = input_item.to_biguint().context("BigInt to BigUint")?;
            let image: Image = input_uint.to_image()?;
            images.push(image);
        }

        // join images
        let result_image: Image;
        if self.is_hstack {
            result_image = Image::hstack(images)?;
        } else {
            result_image = Image::vstack(images)?;
        }
        let output_uint: BigUint = result_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImagePopularColorFunction {
    id: u32,
    outputs: u8,
    is_popular: bool,
}

impl ImagePopularColorFunction {
    fn popular(id: u32, outputs: u8) -> Self {
        Self {
            id,
            outputs,
            is_popular: true,
        }
    }

    fn unpopular(id: u32, outputs: u8) -> Self {
        Self {
            id,
            outputs,
            is_popular: false,
        }
    }
}

impl UnofficialFunction for ImagePopularColorFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: self.outputs }
    }

    fn name(&self) -> String {
        if self.is_popular {
            return format!("Image the {} most popular colors, sorted by popularity", self.outputs);
        } else {
            return format!("Image the {} most unpopular colors, sorted by unpopularity", self.outputs);
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        let histogram: Histogram = image.histogram_all();
        let pairs: Vec<(u32, u8)>;
        if self.is_popular {
            pairs = histogram.pairs_descending();
        } else {
            pairs = histogram.pairs_ascending();
        }
        let mut colors: Vec<i32> = pairs.iter().map(|(_, color)| (*color) as i32).collect();

        // Take N of the most popular colors
        colors.truncate(self.outputs as usize);

        // Pad with -1
        while colors.len() < (self.outputs as usize) {
            colors.push(-1);
        }

        // Convert to BigInt's
        let mut output_vec = Vec::<BigInt>::with_capacity(self.outputs as usize);
        for color in colors {
            let color_bigint: BigInt = color.to_bigint().context("i32 to BigInt")?;
            output_vec.push(color_bigint);
        }
        Ok(output_vec)
    }
}

struct ImageOverlayAnotherImageByColorMaskFunction {
    id: u32,
}

impl ImageOverlayAnotherImageByColorMaskFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageOverlayAnotherImageByColorMaskFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: Overlay another image by using a color as mask".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image0: Image = input0_uint.to_image()?;

        // input1 is image
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let image1: Image = input1_uint.to_image()?;

        // input2 is pixel_color 
        let pixel_color: u8 = input[2].to_u8().context("u8 pixel_color")?;

        let output_image: Image = image0.overlay_with_mask_color(&image1, pixel_color)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageOverlayAnotherImageAtPositionFunction {
    id: u32,
}

impl ImageOverlayAnotherImageAtPositionFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageOverlayAnotherImageAtPositionFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 4, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: Overlay another image at position (x, y)".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 4 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image0: Image = input0_uint.to_image()?;

        // input1 is image
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let image1: Image = input1_uint.to_image()?;

        // input2 is position x
        let position_x: i32 = input[2].to_i32().context("i32 position x")?;

        // input3 is position y
        let position_y: i32 = input[3].to_i32().context("i32 position y")?;

        let output_image: Image = image0.overlay_with_position(&image1, position_x, position_y)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageOutlineFunction {
    id: u32,
}

impl ImageOutlineFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageOutlineFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: Draw outline around things that aren't the background".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is pixel_color 
        let outline_color: u8 = input[1].to_u8().context("u8 pixel_color")?;

        // input2 is pixel_color 
        let background_color: u8 = input[2].to_u8().context("u8 pixel_color")?;

        let output_image: Image = image.outline_type1(outline_color, background_color)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageDenoiseType1Function {
    id: u32,
}

impl ImageDenoiseType1Function {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageDenoiseType1Function {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Denoise type1. denoise noisy pixels. Takes a 2nd parameter: background color.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is pixel_color 
        let background_color: u8 = input[1].to_u8().context("u8 pixel_color")?;

        let output_image: Image = image.denoise_type1(background_color)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageDenoiseType2Function {
    id: u32,
}

impl ImageDenoiseType2Function {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageDenoiseType2Function {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Denoise type2. denoise noisy pixels. Takes a 2nd parameter: noise color.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is pixel_color 
        let noise_color: u8 = input[1].to_u8().context("u8 pixel_color")?;

        let output_image: Image = image.denoise_type2(noise_color)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageDenoiseType3Function {
    id: u32,
}

impl ImageDenoiseType3Function {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageDenoiseType3Function {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Denoise type3. denoise noisy pixels. Takes a 2nd parameter: number of repair iterations.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is repair_iterations 
        let repair_iterations: u8 = input[1].to_u8().context("u8 repair_iterations")?;

        let output_image: Image = image.denoise_type3(repair_iterations)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageNoiseColorFunction {
    id: u32,
    outputs: u8,
}

impl ImageNoiseColorFunction {
    fn new(id: u32, outputs: u8) -> Self {
        Self {
            id,
            outputs,
        }
    }
}

impl UnofficialFunction for ImageNoiseColorFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: self.outputs }
    }

    fn name(&self) -> String {
        format!("Extract from (noisy image, denoised image), the {} most noisy colors, sorted by popularity", self.outputs)
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let noisy_image: Image = input0_uint.to_image()?;

        // input1 is image
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let denoised_image: Image = input1_uint.to_image()?;

        let noise_color_vec: Vec<u8> = noisy_image.noise_color_vec(&denoised_image)?;
        let mut colors: Vec<i32> = noise_color_vec.iter().map(|color| *color as i32).collect();

        // Take N of the most popular colors
        colors.truncate(self.outputs as usize);

        // Pad with -1
        while colors.len() < (self.outputs as usize) {
            colors.push(-1);
        }

        // Convert to BigInt's
        let mut output_vec = Vec::<BigInt>::with_capacity(self.outputs as usize);
        for color in colors {
            let color_bigint: BigInt = color.to_bigint().context("i32 to BigInt")?;
            output_vec.push(color_bigint);
        }
        Ok(output_vec)
    }
}

struct ImageDetectHoleFunction {
    id: u32,
}

impl ImageDetectHoleFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageDetectHoleFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: detect holes. Takes a color parameter for the empty areas.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is pixel_color 
        let empty_color: u8 = input[1].to_u8().context("u8 pixel_color")?;

        let output_image: Image = image.detect_hole_type1(empty_color)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageRemoveGridFunction {
    id: u32,
}

impl ImageRemoveGridFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageRemoveGridFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: remove grid patterns.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        let output_image: Image = image.remove_grid()?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

enum ImageBuildPaletteMapFunctionMode {
    HistogramBased,
    ColorSymmetryBased,
}

struct ImageBuildPaletteMapFunction {
    id: u32,
    reverse: bool,
    mode: ImageBuildPaletteMapFunctionMode,
}

impl ImageBuildPaletteMapFunction {
    fn new(id: u32, reverse: bool, mode: ImageBuildPaletteMapFunctionMode) -> Self {
        Self {
            id,
            reverse,
            mode,
        }
    }
}

impl UnofficialFunction for ImageBuildPaletteMapFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        let suffix: String;
        if self.reverse {
            suffix = " The mapping is reversed".to_string();
        } else {
            suffix = " The mapping is forward".to_string();
        }
        match self.mode {
            ImageBuildPaletteMapFunctionMode::HistogramBased => {
                return format!("Construct a color mapping from one image to another image, based on histogram. Both images must have the same number of unique colors.{}", suffix);
            },
            ImageBuildPaletteMapFunctionMode::ColorSymmetryBased => {
                return format!("Construct a color mapping from one image to another image, based on color symmetry.{}", suffix);
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image0: Image = input0_uint.to_image()?;

        // input1 is image
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let image1: Image = input1_uint.to_image()?;

        let output_image: Image;
        match self.mode {
            ImageBuildPaletteMapFunctionMode::HistogramBased => {
                output_image = image0.palette_using_histogram(&image1, self.reverse)?;
            },
            ImageBuildPaletteMapFunctionMode::ColorSymmetryBased => {
                output_image = image0.palette_using_color_symmetry(&image1, self.reverse)?;
            },
        }

        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

#[derive(Debug)]
enum ImageExtractRowColumnFunctionMode {
    GetTop,
    GetBottom,
    GetLeft,
    GetRight,
    RemoveTop,
    RemoveBottom,
    RemoveLeft,
    RemoveRight,
}

struct ImageExtractRowColumnFunction {
    id: u32,
    mode: ImageExtractRowColumnFunctionMode,
}

impl ImageExtractRowColumnFunction {
    fn new(id: u32, mode: ImageExtractRowColumnFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageExtractRowColumnFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImageExtractRowColumnFunctionMode::GetTop => {
                return "get N top rows".to_string();
            },
            ImageExtractRowColumnFunctionMode::GetBottom => {
                return "get N bottom rows".to_string();
            },
            ImageExtractRowColumnFunctionMode::GetLeft => {
                return "get N left columns".to_string();
            },
            ImageExtractRowColumnFunctionMode::GetRight => {
                return "get N right columns".to_string();
            },
            ImageExtractRowColumnFunctionMode::RemoveTop => {
                return "remove N top rows".to_string();
            },
            ImageExtractRowColumnFunctionMode::RemoveBottom => {
                return "remove N bottom rows".to_string();
            },
            ImageExtractRowColumnFunctionMode::RemoveLeft => {
                return "remove N left columns".to_string();
            },
            ImageExtractRowColumnFunctionMode::RemoveRight => {
                return "remove N right columns".to_string();
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let mut image: Image = input0_uint.to_image()?;

        // input1 is number of rows/columns
        let n: u8 = input[1].to_u8().context("u8 padding_count")?;

        match self.mode {
            ImageExtractRowColumnFunctionMode::GetTop => {
                image = image.top_rows(n)?;
            },
            ImageExtractRowColumnFunctionMode::GetBottom => {
                image = image.bottom_rows(n)?;
            },
            ImageExtractRowColumnFunctionMode::GetLeft => {
                image = image.left_columns(n)?;
            },
            ImageExtractRowColumnFunctionMode::GetRight => {
                image = image.right_columns(n)?;
            },
            ImageExtractRowColumnFunctionMode::RemoveTop => {
                image = image.remove_top_rows(n)?;
            },
            ImageExtractRowColumnFunctionMode::RemoveBottom => {
                image = image.remove_bottom_rows(n)?;
            },
            ImageExtractRowColumnFunctionMode::RemoveLeft => {
                image = image.remove_left_columns(n)?;
            },
            ImageExtractRowColumnFunctionMode::RemoveRight => {
                image = image.remove_right_columns(n)?;
            },
        }
        let output_uint: BigUint = image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageHistogramFunction {
    id: u32,
}

impl ImageHistogramFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageHistogramFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        "Histogram of image. The most popular to the left, least popular to the right. The top row is the counters. The bottom row is the colors.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        let histogram: Histogram = image.histogram_all();
        let image: Image = histogram.to_image()?;

        let output_uint: BigUint = image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageHistogramWithMaskFunction {
    id: u32,
}

impl ImageHistogramWithMaskFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageHistogramWithMaskFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Histogram of image using a mask. Only where the mask is non-zero, are the image pixels added to the histogram.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is image
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let mask: Image = input1_uint.to_image()?;

        let histogram: Histogram = image.histogram_with_mask(&mask)?;
        let output_image: Image = histogram.to_image()?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageNumberOfUniqueColorsFunction {
    id: u32,
}

impl ImageNumberOfUniqueColorsFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageNumberOfUniqueColorsFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        "Number of unique colors in image.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        let histogram: Histogram = image.histogram_all();
        let color_count: u32 = histogram.number_of_counters_greater_than_zero();

        let output: BigInt = color_count.to_bigint().context("u32 to BigInt")?;
        Ok(vec![output])
    }
}

#[derive(Debug)]
enum ImageToMaskFunctionFunctionMode {
    WhereColorIs,
    WhereColorIsDifferent,
}

struct ImageToMaskFunction {
    id: u32,
    mode: ImageToMaskFunctionFunctionMode,
}

impl ImageToMaskFunction {
    fn new(id: u32, mode: ImageToMaskFunctionFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageToMaskFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImageToMaskFunctionFunctionMode::WhereColorIs => {
                return "Convert to a mask image by converting `color` to 1 and converting anything else to to 0.".to_string();
            },
            ImageToMaskFunctionFunctionMode::WhereColorIsDifferent => {
                return "Convert to a mask image by converting `color` to 0 and converting anything else to to 1.".to_string();
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 2 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        // input1 is pixel_color 
        let color: u8 = input[1].to_u8().context("u8 pixel_color")?;

        let output_image: Image;
        match self.mode {
            ImageToMaskFunctionFunctionMode::WhereColorIs => {
                output_image = image.to_mask_where_color_is(color);
            },
            ImageToMaskFunctionFunctionMode::WhereColorIsDifferent => {
                output_image = image.to_mask_where_color_is_different(color);
            },
        }
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageInvertMaskFunction {
    id: u32,
}

impl ImageInvertMaskFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageInvertMaskFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        "Inverts a mask image by converting 0 to 1 and converting [1..255] to 0.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        let output_image: Image = image.invert_mask();
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

enum ImagePopularObjectFunctionMode {
    MostPopular,
    LeastPopular,
}

struct ImagePopularObjectFunction {
    id: u32,
    mode: ImagePopularObjectFunctionMode,
}

impl ImagePopularObjectFunction {
    fn new(id: u32, mode: ImagePopularObjectFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImagePopularObjectFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImagePopularObjectFunctionMode::MostPopular => {
                return "Image: Extracts the most popular object.".to_string()
            },
            ImagePopularObjectFunctionMode::LeastPopular => {
                return "Image: Extracts the least popular object.".to_string()
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 1 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image: Image = input0_uint.to_image()?;

        let output_image: Image;
        match self.mode {
            ImagePopularObjectFunctionMode::MostPopular => {
                output_image = PopularObjects::most_popular_object(&image)?;
            },
            ImagePopularObjectFunctionMode::LeastPopular => {
                output_image = PopularObjects::least_popular_object(&image)?;
            },
        }
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

#[derive(Debug)]
enum ImageNeighbourFunctionMode {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

struct ImageNeighbourFunction {
    id: u32,
    mode: ImageNeighbourFunctionMode,
}

impl ImageNeighbourFunction {
    fn new(id: u32, mode: ImageNeighbourFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageNeighbourFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        match self.mode {
            ImageNeighbourFunctionMode::Up => {
                return "color of nearest neighbour pixel 'up'".to_string();
            },
            ImageNeighbourFunctionMode::Down => {
                return "color of nearest neighbour pixel 'down'".to_string();
            },
            ImageNeighbourFunctionMode::Left => {
                return "color of nearest neighbour pixel 'left'".to_string();
            },
            ImageNeighbourFunctionMode::Right => {
                return "color of nearest neighbour pixel 'right'".to_string();
            },
            ImageNeighbourFunctionMode::UpLeft => {
                return "color of nearest neighbour pixel 'up left'".to_string();
            },
            ImageNeighbourFunctionMode::UpRight => {
                return "color of nearest neighbour pixel 'up right'".to_string();
            },
            ImageNeighbourFunctionMode::DownLeft => {
                return "color of nearest neighbour pixel 'down left'".to_string();
            },
            ImageNeighbourFunctionMode::DownRight => {
                return "color of nearest neighbour pixel 'down right'".to_string();
            },
        }
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let mut image: Image = input0_uint.to_image()?;

        // input1 is ignore_mask
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let ignore_mask: Image = input1_uint.to_image()?;

        // input2 is color_when_there_is_no_neighbour
        let color_when_there_is_no_neighbour: u8 = input[2].to_u8().context("u8 color_when_there_is_no_neighbour")?;

        let direction: ImageNeighbourDirection = match self.mode {
            ImageNeighbourFunctionMode::Up => ImageNeighbourDirection::Up,
            ImageNeighbourFunctionMode::Down => ImageNeighbourDirection::Down,
            ImageNeighbourFunctionMode::Left => ImageNeighbourDirection::Left,
            ImageNeighbourFunctionMode::Right => ImageNeighbourDirection::Right,
            ImageNeighbourFunctionMode::UpLeft => ImageNeighbourDirection::UpLeft,
            ImageNeighbourFunctionMode::UpRight => ImageNeighbourDirection::UpRight,
            ImageNeighbourFunctionMode::DownLeft => ImageNeighbourDirection::DownLeft,
            ImageNeighbourFunctionMode::DownRight => ImageNeighbourDirection::DownRight,
        };
        image = image.neighbour_color(&ignore_mask, direction, color_when_there_is_no_neighbour)?;
        let output_uint: BigUint = image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

struct ImageSetPixelWhereTwoImagesAgreeFunction {
    id: u32,
}

impl ImageSetPixelWhereTwoImagesAgreeFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageSetPixelWhereTwoImagesAgreeFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 4, outputs: 1 }
    }

    fn name(&self) -> String {
        "Set pixel where two images agree on the pixel value.".to_string()
    }

    fn run(&self, input: Vec<BigInt>) -> anyhow::Result<Vec<BigInt>> {
        if input.len() != 4 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let image0: Image = input0_uint.to_image()?;

        // input1 is image
        if input[1].is_negative() {
            return Err(anyhow::anyhow!("Input[1] must be non-negative"));
        }
        let input1_uint: BigUint = input[1].to_biguint().context("BigInt to BigUint")?;
        let image1: Image = input1_uint.to_image()?;

        // input2 is image
        if input[2].is_negative() {
            return Err(anyhow::anyhow!("Input[2] must be non-negative"));
        }
        let input2_uint: BigUint = input[2].to_biguint().context("BigInt to BigUint")?;
        let image2: Image = input2_uint.to_image()?;

        // input3 is the color to ignore
        let color_must_be_different_than: u8 = input[3].to_u8().context("Input[3] u8 pixel_color")?;

        let mut output_image: Image = image0;
        output_image.set_pixel_where_two_images_agree(&image1, &image2, color_must_be_different_than)?;

        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

#[allow(dead_code)]
pub fn register_arc_functions(registry: &UnofficialFunctionRegistry) {
    macro_rules! register_function {
        ($create_instance:expr) => {
            registry.register(Arc::new(Box::new($create_instance)));
        }
    }
    
    // Developer tools
    register_function!(ImageDebugFunction::new(100000));

    // Basic
    register_function!(ImageGetAttributeFunction::new(101000, ImageGetAttributeFunctionMode::Width));
    register_function!(ImageGetAttributeFunction::new(101001, ImageGetAttributeFunctionMode::Height));
    register_function!(ImageGetPixelFunction::new(101002));
    register_function!(ImageSetPixelFunction::new(101003));

    // Create image
    register_function!(ImageWithColorFunction::new(101010));

    // Image horizontal stack
    for n in 2..=9 {
        register_function!(ImageStackFunction::hstack(101030, n));
    }

    // Image vertical stack
    for n in 2..=9 {
        register_function!(ImageStackFunction::vstack(101040, n));
    }
    
    // Replace color
    register_function!(ImageReplaceColorFunction::new(101050, ImageReplaceColorFunctionMode::ReplaceColor));
    register_function!(ImageReplaceColorFunction::new(101051, ImageReplaceColorFunctionMode::ReplaceColorsOtherThan));
    register_function!(ImageReplaceColorsWithPaletteImageFunction::new(101052));

    // Extract popular colors
    for n in 1..=9 {
        register_function!(ImagePopularColorFunction::popular(101060, n));
    }

    // Extract unpopular colors
    for n in 1..=9 {
        register_function!(ImagePopularColorFunction::unpopular(101070, n));
    }
    
    // Draw outline
    register_function!(ImageOutlineFunction::new(101080));
    
    // Denoise
    register_function!(ImageDenoiseType1Function::new(101090));
    register_function!(ImageDenoiseType2Function::new(101091));
    register_function!(ImageDenoiseType3Function::new(101092));

    // Extract noise colors from (noise image, denoised image)
    for n in 1..=9 {
        register_function!(ImageNoiseColorFunction::new(101100, n));
    }

    // Detect hole
    register_function!(ImageDetectHoleFunction::new(101110));

    // Remove grid
    register_function!(ImageRemoveGridFunction::new(101120));

    // Color mapping from one image to another image
    register_function!(ImageBuildPaletteMapFunction::new(101130, false, ImageBuildPaletteMapFunctionMode::HistogramBased));
    register_function!(ImageBuildPaletteMapFunction::new(101131, true, ImageBuildPaletteMapFunctionMode::HistogramBased));
    register_function!(ImageBuildPaletteMapFunction::new(101132, false, ImageBuildPaletteMapFunctionMode::ColorSymmetryBased));
    register_function!(ImageBuildPaletteMapFunction::new(101133, true, ImageBuildPaletteMapFunctionMode::ColorSymmetryBased));

    // Remove duplicates
    register_function!(ImageRemoveDuplicatesFunction::new(101140, ImageRemoveDuplicatesFunctionMode::RowsAndColumns));
    register_function!(ImageRemoveDuplicatesFunction::new(101141, ImageRemoveDuplicatesFunctionMode::Rows));
    register_function!(ImageRemoveDuplicatesFunction::new(101142, ImageRemoveDuplicatesFunctionMode::Columns));

    // Overlay by color
    register_function!(ImageOverlayAnotherImageByColorMaskFunction::new(101150));
    register_function!(ImageOverlayAnotherImageAtPositionFunction::new(101151));

    // Trim
    register_function!(ImageTrimFunction::new(101160));

    // Rotate
    register_function!(ImageRotateFunction::new(101170));

    // Offset
    register_function!(ImageOffsetFunction::new(101180, ImageOffsetFunctionMode::Wrap));
    register_function!(ImageOffsetFunction::new(101181, ImageOffsetFunctionMode::Clamp));

    // Flip
    register_function!(ImageFlipFunction::new(101190, ImageFlipFunctionMode::FlipX));
    register_function!(ImageFlipFunction::new(101191, ImageFlipFunctionMode::FlipY));
    register_function!(ImageFlipFunction::new(101192, ImageFlipFunctionMode::FlipXY));

    // Image resize
    register_function!(ImageResizeFunction::new(101200));

    // Padding
    register_function!(ImagePaddingFunction::new(101210, ImagePaddingFunctionMode::Top));
    register_function!(ImagePaddingFunction::new(101211, ImagePaddingFunctionMode::Bottom));
    register_function!(ImagePaddingFunction::new(101212, ImagePaddingFunctionMode::Left));
    register_function!(ImagePaddingFunction::new(101213, ImagePaddingFunctionMode::Right));

    // Extract N rows/columns
    register_function!(ImageExtractRowColumnFunction::new(101220, ImageExtractRowColumnFunctionMode::GetTop));
    register_function!(ImageExtractRowColumnFunction::new(101221, ImageExtractRowColumnFunctionMode::GetBottom));
    register_function!(ImageExtractRowColumnFunction::new(101222, ImageExtractRowColumnFunctionMode::GetLeft));
    register_function!(ImageExtractRowColumnFunction::new(101223, ImageExtractRowColumnFunctionMode::GetRight));
    register_function!(ImageExtractRowColumnFunction::new(101224, ImageExtractRowColumnFunctionMode::RemoveTop));
    register_function!(ImageExtractRowColumnFunction::new(101225, ImageExtractRowColumnFunctionMode::RemoveBottom));
    register_function!(ImageExtractRowColumnFunction::new(101226, ImageExtractRowColumnFunctionMode::RemoveLeft));
    register_function!(ImageExtractRowColumnFunction::new(101227, ImageExtractRowColumnFunctionMode::RemoveRight));
    
    // Histogram
    register_function!(ImageHistogramFunction::new(101230));
    register_function!(ImageHistogramWithMaskFunction::new(101231));

    // Unique colors
    register_function!(ImageNumberOfUniqueColorsFunction::new(101240));

    // Mask
    register_function!(ImageToMaskFunction::new(101250, ImageToMaskFunctionFunctionMode::WhereColorIs));
    register_function!(ImageToMaskFunction::new(101251, ImageToMaskFunctionFunctionMode::WhereColorIsDifferent));
    register_function!(ImageInvertMaskFunction::new(101252));

    // Objects
    register_function!(ImagePopularObjectFunction::new(102000, ImagePopularObjectFunctionMode::MostPopular));
    register_function!(ImagePopularObjectFunction::new(102001, ImagePopularObjectFunctionMode::LeastPopular));

    // Color of neighbour pixel
    register_function!(ImageNeighbourFunction::new(102060, ImageNeighbourFunctionMode::Up));
    register_function!(ImageNeighbourFunction::new(102061, ImageNeighbourFunctionMode::Down));
    register_function!(ImageNeighbourFunction::new(102062, ImageNeighbourFunctionMode::Left));
    register_function!(ImageNeighbourFunction::new(102063, ImageNeighbourFunctionMode::Right));
    register_function!(ImageNeighbourFunction::new(102064, ImageNeighbourFunctionMode::UpLeft));
    register_function!(ImageNeighbourFunction::new(102065, ImageNeighbourFunctionMode::UpRight));
    register_function!(ImageNeighbourFunction::new(102066, ImageNeighbourFunctionMode::DownLeft));
    register_function!(ImageNeighbourFunction::new(102067, ImageNeighbourFunctionMode::DownRight));

    // Set pixel where two images agree
    register_function!(ImageSetPixelWhereTwoImagesAgreeFunction::new(102100));
}
