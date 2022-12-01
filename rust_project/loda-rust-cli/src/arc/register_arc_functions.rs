use super::{Bitmap, BitmapToNumber, NumberToBitmap, BitmapOffset};
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
        if input.len() == 3 {
            return Err(anyhow::anyhow!("Wrong number of inputs"));
        }

        // input0 is image
        if input[0].is_negative() {
            return Err(anyhow::anyhow!("Input[0] must be non-negative"));
        }
        let input0_uint: BigUint = input[0].to_biguint().context("BigInt to BigUint")?;
        let input_image: Bitmap = input0_uint.to_bitmap()?;

        // input1 is dx
        let dx: i32 = input[1].to_i32().context("to_i32 dx")?;

        // input2 is dy
        let dy: i32 = input[2].to_i32().context("to_i32 dy")?;

        let output_image: Bitmap = input_image.offset_wrap(dx, dy)?;
        let output_uint: BigUint = output_image.to_number()?;
        let output: BigInt = output_uint.to_bigint().context("BigUint to BigInt")?;
        Ok(vec![output])
    }
}


pub fn register_arc_functions(registry: &UnofficialFunctionRegistry) {
    registry.register(Arc::new(Box::new(ImageOffsetFunction::new(100001))));
}
