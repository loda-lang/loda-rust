use super::arc_work_model::{Task, PairType};
use super::Image;
use crate::config::Config;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

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
            if y == 0 {
                s += "c";
            } else {
                s += ",b";
            }
            for x in 0..image.width() {
                s += ",";
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
