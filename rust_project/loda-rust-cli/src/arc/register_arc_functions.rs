use super::{Image, ImageToNumber, NumberToImage, ImageOffset, ImageTrim, ImageRemoveDuplicates, ImageRotate};
use super::{ImageHistogram, ImageReplaceColor, ImageSymmetry, ImagePadding, ImageResize, ImageStack};
use super::{Histogram, ImageOverlay, ImageOutline, ImageDenoise, ImageNoiseColor, ImageDetectHole};
use super::{ImageRemoveGrid, PaletteImage};
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
        println!("image: {:?}", input_image);

        // no output
        Ok(vec!())
    }
}

struct ImageOffsetFunction {
    id: u32,
}

impl ImageOffsetFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageOffsetFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        "Adjust image offset(dx, dy) with wrap".to_string()
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

        let output_image: Image = input_image.offset_wrap(dx, dy)?;
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

struct ImageRemoveDuplicatesFunction {
    id: u32,
}

impl ImageRemoveDuplicatesFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageRemoveDuplicatesFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: Remove duplicate rows/columns".to_string()
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

        let output_image: Image = input_image.remove_duplicates()?;
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
    Even,
    TopBottom,
    LeftRight,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
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
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        format!("ImagePaddingFunction {:?} pad by one pixel with color", self.mode)
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

        // input1 is pixel_color 
        let pixel_color: u8 = input[1].to_u8().context("u8 pixel_color")?;

        match self.mode {
            ImagePaddingFunctionMode::Even => {
                image = image.padding_advanced(1, 1, 1, 1, pixel_color)?;
            },
            ImagePaddingFunctionMode::TopBottom => {
                image = image.padding_advanced(1, 0, 0, 1, pixel_color)?;
            },
            ImagePaddingFunctionMode::LeftRight => {
                image = image.padding_advanced(0, 1, 1, 0, pixel_color)?;
            },
            ImagePaddingFunctionMode::TopLeft => {
                image = image.padding_advanced(1, 1, 0, 0, pixel_color)?;
            },
            ImagePaddingFunctionMode::TopRight => {
                image = image.padding_advanced(1, 0, 1, 0, pixel_color)?;
            },
            ImagePaddingFunctionMode::BottomLeft => {
                image = image.padding_advanced(0, 1, 0, 1, pixel_color)?;
            },
            ImagePaddingFunctionMode::BottomRight => {
                image = image.padding_advanced(0, 0, 1, 1, pixel_color)?;
            },
        }
        let output_uint: BigUint = image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

#[derive(Debug)]
enum ImageResizeFunctionMode {
    XYMul2,
    XYMul3,
    XYDiv2,
}

struct ImageResizeFunction {
    id: u32,
    mode: ImageResizeFunctionMode,
}

impl ImageResizeFunction {
    fn new(id: u32, mode: ImageResizeFunctionMode) -> Self {
        Self {
            id,
            mode,
        }
    }
}

impl UnofficialFunction for ImageResizeFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 1 }
    }

    fn name(&self) -> String {
        format!("ImageResizeFunction {:?}", self.mode)
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
            ImageResizeFunctionMode::XYMul2 => {
                image = image.resize(image.width() * 2, image.height() * 2)?;
            },
            ImageResizeFunctionMode::XYMul3 => {
                image = image.resize(image.width() * 3, image.height() * 3)?;
            },
            ImageResizeFunctionMode::XYDiv2 => {
                image = image.resize(image.width() / 2, image.height() / 2)?;
            },
        }
        let output_uint: BigUint = image.to_number()?;
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

struct ImageDenoiseFunction {
    id: u32,
}

impl ImageDenoiseFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageDenoiseFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: denoise noisy pixels. Takes a background color parameter.".to_string()
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

struct ImageBuildPaletteMapFunction {
    id: u32,
    reverse: bool,
}

impl ImageBuildPaletteMapFunction {
    fn new(id: u32, reverse: bool) -> Self {
        Self {
            id,
            reverse,
        }
    }
}

impl UnofficialFunction for ImageBuildPaletteMapFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 2, outputs: 1 }
    }

    fn name(&self) -> String {
        if self.reverse {
            return "Construct a reverse color mapping from one image to another image. Both images must have the same number of unique colors.".to_string()
        } else {
            return "Construct a forward color mapping from one image to another image. Both images must have the same number of unique colors.".to_string()
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

        let output_image: Image = PaletteImage::palette_image(&image0, &image1, self.reverse)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}

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
    register_function!(ImageDenoiseFunction::new(101090));

    // Extract noise colors from (noise image, denoised image)
    for n in 1..=9 {
        register_function!(ImageNoiseColorFunction::new(101100, n));
    }

    // Detect hole
    register_function!(ImageDetectHoleFunction::new(101110));

    // Remove grid
    register_function!(ImageRemoveGridFunction::new(101120));

    // Color mapping from one image to another image
    register_function!(ImageBuildPaletteMapFunction::new(101130, false));
    register_function!(ImageBuildPaletteMapFunction::new(101131, true));

    // Remove duplicates
    register_function!(ImageRemoveDuplicatesFunction::new(101140));

    // Overlay by color
    register_function!(ImageOverlayAnotherImageByColorMaskFunction::new(101150));

    // Trim
    register_function!(ImageTrimFunction::new(101160));

    // Rotate
    register_function!(ImageRotateFunction::new(101170));

    // Offset
    register_function!(ImageOffsetFunction::new(101180));

    // Flip
    register_function!(ImageFlipFunction::new(101190, ImageFlipFunctionMode::FlipX));
    register_function!(ImageFlipFunction::new(101191, ImageFlipFunctionMode::FlipY));
    register_function!(ImageFlipFunction::new(101192, ImageFlipFunctionMode::FlipXY));

    // Image resize
    register_function!(ImageResizeFunction::new(101200, ImageResizeFunctionMode::XYMul2));
    register_function!(ImageResizeFunction::new(101201, ImageResizeFunctionMode::XYMul3));
    register_function!(ImageResizeFunction::new(101202, ImageResizeFunctionMode::XYDiv2));

    // Padding top/bottom, left/right
    register_function!(ImagePaddingFunction::new(101220, ImagePaddingFunctionMode::TopBottom));
    register_function!(ImagePaddingFunction::new(101221, ImagePaddingFunctionMode::LeftRight));

    // Padding in corners
    register_function!(ImagePaddingFunction::new(101230, ImagePaddingFunctionMode::TopLeft));
    register_function!(ImagePaddingFunction::new(101231, ImagePaddingFunctionMode::TopRight));
    register_function!(ImagePaddingFunction::new(101232, ImagePaddingFunctionMode::BottomLeft));
    register_function!(ImagePaddingFunction::new(101233, ImagePaddingFunctionMode::BottomRight));

    // Padding evenly
    register_function!(ImagePaddingFunction::new(101240, ImagePaddingFunctionMode::Even));
}
