use std::path::PathBuf;
use std::fs;

pub fn read_testdata(name: &str) -> anyhow::Result<String> {
    let e = env!("CARGO_MANIFEST_DIR");
    let relative_path = format!("src/arc/testdata/{}.json", name);
    let path = PathBuf::from(e).join(relative_path);
    let json_string: String = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) => {
            return Err(anyhow::anyhow!("cannot load file for name {:?}, error: {:?} path: {:?}", name, error, path));
        }
    };
    Ok(json_string)
}
