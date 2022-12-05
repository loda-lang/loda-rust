use super::{Image, ImageToNumber, NumberToImage, ImageOffset, ImageTrim, ImageRemoveDuplicates, ImageRotate};
use super::{ImageHistogram, ImageReplaceColor, ImageSymmetry, ImagePadding, ImageResize, ImageStack};
use super::{Histogram, ImageOverlay};
use loda_rust_core::unofficial_function::{UnofficialFunction, UnofficialFunctionId, UnofficialFunctionRegistry};
use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::{Signed, ToPrimitive};
use std::sync::Arc;
use anyhow::Context;

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

struct ImageGetSizeFunction {
    id: u32,
}

impl ImageGetSizeFunction {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageGetSizeFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 1, outputs: 2 }
    }

    fn name(&self) -> String {
        "Image: get size of image -> (width, height)".to_string()
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

        // Return (width, height)
        let width: BigInt = image.width().to_bigint().context("u8 to BigInt")?;
        let height: BigInt = image.height().to_bigint().context("u8 to BigInt")?;
        Ok(vec![width, height])
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


pub fn register_arc_functions(registry: &UnofficialFunctionRegistry) {
    registry.register(Arc::new(Box::new(ImageOffsetFunction::new(100001))));
    registry.register(Arc::new(Box::new(ImageRotateFunction::new(100002))));
    registry.register(Arc::new(Box::new(ImageTrimFunction::new(100003))));
    registry.register(Arc::new(Box::new(ImageRemoveDuplicatesFunction::new(100004))));
    registry.register(Arc::new(Box::new(ImageOverlayAnotherImageByColorMaskFunction::new(100005))));
    registry.register(Arc::new(Box::new(ImageWithColorFunction::new(100006))));
    registry.register(Arc::new(Box::new(ImageSetPixelFunction::new(100007))));
    registry.register(Arc::new(Box::new(ImageGetPixelFunction::new(100008))));
    registry.register(Arc::new(Box::new(ImageGetSizeFunction::new(100009))));

    // Flip
    registry.register(Arc::new(Box::new(
        ImageFlipFunction::new(100010, ImageFlipFunctionMode::FlipX
    ))));
    registry.register(Arc::new(Box::new(
        ImageFlipFunction::new(100011, ImageFlipFunctionMode::FlipY
    ))));
    registry.register(Arc::new(Box::new(
        ImageFlipFunction::new(100012, ImageFlipFunctionMode::FlipXY
    ))));

    // Padding
    registry.register(Arc::new(Box::new(
        ImagePaddingFunction::new(100013, ImagePaddingFunctionMode::Even
    ))));
    registry.register(Arc::new(Box::new(
        ImagePaddingFunction::new(100014, ImagePaddingFunctionMode::TopBottom
    ))));
    registry.register(Arc::new(Box::new(
        ImagePaddingFunction::new(100015, ImagePaddingFunctionMode::LeftRight
    ))));
    registry.register(Arc::new(Box::new(
        ImagePaddingFunction::new(100016, ImagePaddingFunctionMode::TopLeft
    ))));
    registry.register(Arc::new(Box::new(
        ImagePaddingFunction::new(100017, ImagePaddingFunctionMode::TopRight
    ))));
    registry.register(Arc::new(Box::new(
        ImagePaddingFunction::new(100018, ImagePaddingFunctionMode::BottomLeft
    ))));
    registry.register(Arc::new(Box::new(
        ImagePaddingFunction::new(100019, ImagePaddingFunctionMode::BottomRight
    ))));

    // Image resize
    registry.register(Arc::new(Box::new(
        ImageResizeFunction::new(100020, ImageResizeFunctionMode::XYMul2
    ))));
    registry.register(Arc::new(Box::new(
        ImageResizeFunction::new(100021, ImageResizeFunctionMode::XYMul3
    ))));
    registry.register(Arc::new(Box::new(
        ImageResizeFunction::new(100022, ImageResizeFunctionMode::XYDiv2
    ))));

    // Image horizontal stack
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100032, 2))));
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100033, 3))));
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100034, 4))));
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100035, 5))));
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100036, 6))));
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100037, 7))));
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100038, 8))));
    registry.register(Arc::new(Box::new(ImageStackFunction::hstack(100039, 9))));

    // Image vertical stack
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100042, 2))));
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100043, 3))));
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100044, 4))));
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100045, 5))));
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100046, 6))));
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100047, 7))));
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100048, 8))));
    registry.register(Arc::new(Box::new(ImageStackFunction::vstack(100049, 9))));
    
    // Replace color
    registry.register(Arc::new(Box::new(
        ImageReplaceColorFunction::new(100050, ImageReplaceColorFunctionMode::ReplaceColor
    ))));
    registry.register(Arc::new(Box::new(
        ImageReplaceColorFunction::new(100051, ImageReplaceColorFunctionMode::ReplaceColorsOtherThan
    ))));

    // Popular colors
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100061, 1))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100062, 2))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100063, 3))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100064, 4))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100065, 5))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100066, 6))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100067, 7))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100068, 8))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::popular(100069, 9))));

    // Unpopular colors
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100071, 1))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100072, 2))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100073, 3))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100074, 4))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100075, 5))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100076, 6))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100077, 7))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100078, 8))));
    registry.register(Arc::new(Box::new(ImagePopularColorFunction::unpopular(100079, 9))));
}
