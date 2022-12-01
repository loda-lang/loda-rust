use super::UnofficialFunction;
use num_bigint::{BigInt, ToBigInt};
use num_traits::ToPrimitive;

pub fn perform_run(f: Box<dyn UnofficialFunction>, input: Vec<i32>) -> anyhow::Result<Vec<i32>> {
    let input_vec: Vec<BigInt> = input.iter().map(|v| v.to_bigint().unwrap() ).collect();
    let output_bigints: Vec<BigInt> = f.run(input_vec)?;
    let output: Vec<i32> = output_bigints.iter().map(|v| v.to_i32().unwrap() ).collect();
    Ok(output)
}
