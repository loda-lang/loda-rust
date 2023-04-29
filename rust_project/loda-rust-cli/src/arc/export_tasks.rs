use super::arc_work_model::{Task, PairType};
use super::Image;
use crate::config::Config;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

/// Take this ARC image:
/// ```
/// [[0, 1, 2],
/// [3, 4, 5],
/// [6, 7, 8]]
/// ```
/// 
/// And convert into this representation:
/// ```
///  aaa
/// b012
/// b345
/// b678
/// ```
/// 
/// The data looks like this.
/// 
/// - The lines alternates between `Input` and `Output`.
/// - The prefix `I` is for `Input`.
/// - The prefix `O` is for `Output`.
/// - The `a` is the top-most row.
/// - The `b` indicates the beginning of row.
/// - The separator `-` is between ARC tasks.
/// - The separator `,` is used between pixels.
/// - Future experiment: add prefix `T` for `Output template`, an image with the predicted size for the output.
/// - Future experiment: add prefix `P` for `Palette colors`, the colors predicted for the output.
/// - Future experiment: add prefix `G` for `Grid`.
/// - Future experiment: add prefix `R` for `Repair`.
/// - Future experiment: add prefix `E` for `Enumerated Objects`.
///
/// Example of the generated file:
/// 
/// ```
/// -
/// I,a2,a2,a2,b,22,21,28,b,22,18,88
/// O,a2,a2,a2,b,22,25,25,b,22,55,55
/// I,a1,a1,a1,b,18,11,13,b,88,12,32
/// O,a1,a1,a1,b,15,11,15,b,55,15,55
/// I,a2,a2,a2,b,28,28,22,b,82,82,22
/// O,a2,a2,a2,b,25,25,22,b,52,52,22
/// I,a3,a3,a8,b,34,34,84,b,48,41,41
/// O,a5,a5,a5,b,54,54,54,b,45,45,45
/// I,a1,a3,a2,b,13,33,22,b,31,33,22
/// O,a5,a3,a5,b,53,33,55,b,35,33,55
/// -
/// I,a8,a8,a0,a0,a0,b,88,88,00,00,00,b,80,80,00,00,00,b,00,00,00,00,00,b,00,00,00,00,00
/// O,a0,a0,a0,a0,a0,b,02,02,00,00,00,b,22,22,00,00,00,b,20,20,00,00,00,b,00,00,00,00,00
/// I,a0,a8,a0,b,00,80,00,b,00,00,00
/// O,a0,a0,a0,b,00,02,00,b,00,20,00
/// I,a0,a0,a0,a0,a0,b,00,08,08,08,00,b,00,80,80,80,00,b,00,00,00,00,00,b,00,00,00,00,00
/// O,a0,a0,a0,a0,a0,b,00,00,00,00,00,b,00,02,02,02,00,b,00,20,20,20,00,b,00,00,00,00,00
/// I,a0,a0,a8,a0,a0,b,00,08,88,00,00,b,00,80,88,00,00,b,00,00,80,00,00,b,00,00,00,00,00
/// O,a0,a0,a0,a0,a0,b,00,00,02,00,00,b,00,02,22,00,00,b,00,20,22,00,00,b,00,00,20,00,00
/// -
/// ```
pub struct ExportTasks;

impl ExportTasks {
    pub fn export(tasks: &Vec<Task>) -> anyhow::Result<()> {
        let config = Config::load();
        let path: PathBuf = config.basedir().join("arc-dataset.txt");
        
        println!("task count: {}", tasks.len());
        let mut s = String::new();
        for task in tasks {
            s += "\n-";
            for pair in &task.pairs {
                s += "\nI,";
                s += &Self::serialize_image(&pair.input.image)?;
                s += "\nO,";
                match pair.pair_type {
                    PairType::Train => {
                        s += &Self::serialize_image(&pair.output.image)?;
                    },
                    PairType::Test => {
                        s += &Self::serialize_image(&pair.output.test_image)?;
                    }
                }
            }
        }

        println!("saving file: {:?}", path);
        let mut file = File::create(&path)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }

    fn serialize_image(image: &Image) -> anyhow::Result<String> {
        let mut s = String::new();
        for y in 0..image.height() {
            if y > 0 {
                s += ",b";
            }
            for x in 0..image.width() {
                if y > 0 || x > 0 {
                    s += ",";
                }
                if y == 0 {
                    s += "a";
                } else {
                    let value: u8 = image.get(x as i32, (y as i32) - 1).unwrap_or(255);
                    if value > 9 {
                        return Err(anyhow::anyhow!("Value is out of range [0..9]"));
                    }
                    s += &format!("{}", value);
                }
                {
                    let value: u8 = image.get(x as i32, y as i32).unwrap_or(255);
                    if value > 9 {
                        return Err(anyhow::anyhow!("Value is out of range [0..9]"));
                    }
                    s += &format!("{}", value);
                }
            }
        }
        Ok(s)
    }
}
