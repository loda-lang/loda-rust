use std::path::PathBuf;
use std::fs;

pub fn path_testdata(name: &str) -> anyhow::Result<PathBuf> {
    let e = env!("CARGO_MANIFEST_DIR");
    let relative_path = format!("src/arc/testdata/{}.json", name);
    let path = PathBuf::from(e).join(relative_path);
    if !path.is_file() {
        return Err(anyhow::anyhow!("missing testdata for name {:?}, path: {:?}", name, path));
    }
    Ok(path)
}

pub fn read_testdata(name: &str) -> anyhow::Result<String> {
    let path: PathBuf = path_testdata(name)?;
    let json_string: String = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) => {
            return Err(anyhow::anyhow!("cannot load file for name {:?}, error: {:?} path: {:?}", name, error, path));
        }
    };
    Ok(json_string)
}
