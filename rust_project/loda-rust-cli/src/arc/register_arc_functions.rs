use super::{Image, ImageToNumber, NumberToImage, ImageOffset, ImageTrim, ImageRemoveDuplicates, ImageRotate, ImageReplaceColor};
use loda_rust_core::unofficial_function::{UnofficialFunction, UnofficialFunctionId, UnofficialFunctionRegistry};
use num_bigint::{BigInt, BigUint, ToBigInt};
use num_traits::{Signed, ToPrimitive};
use std::sync::Arc;
use anyhow::Context;

pub struct ImageOffsetFunction {
    id: u32,
}

impl ImageOffsetFunction {
    pub fn new(id: u32) -> Self {
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

pub struct ImageRotateFunction {
    id: u32,
}

impl ImageRotateFunction {
    pub fn new(id: u32) -> Self {
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

pub struct ImageTrimFunction {
    id: u32,
}

impl ImageTrimFunction {
    pub fn new(id: u32) -> Self {
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

pub struct ImageRemoveDuplicatesFunction {
    id: u32,
}

impl ImageRemoveDuplicatesFunction {
    pub fn new(id: u32) -> Self {
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


pub struct ImageReplaceColorFunction {
    id: u32,
}

impl ImageReplaceColorFunction {
    pub fn new(id: u32) -> Self {
        Self {
            id,
        }
    }
}

impl UnofficialFunction for ImageReplaceColorFunction {
    fn id(&self) -> UnofficialFunctionId {
        UnofficialFunctionId::InputOutput { id: self.id, inputs: 3, outputs: 1 }
    }

    fn name(&self) -> String {
        "Image: replace color x with color y".to_string()
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
        let from_color: u8 = input[1].to_u8().context("u8 from_color")?;

        // input2 is color y
        let to_color: u8 = input[2].to_u8().context("u8 to_color")?;

        let output_image: Image = input_image.replace_color(from_color, to_color)?;
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
    registry.register(Arc::new(Box::new(ImageReplaceColorFunction::new(100005))));
}
