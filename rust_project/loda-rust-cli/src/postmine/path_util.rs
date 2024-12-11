use std::path::{Path, PathBuf};

pub struct PathUtil;

impl PathUtil {
    fn path_with_status_extension(path: &Path, statusname: &str) -> PathBuf {
        let mut result = PathBuf::from(path);
        if let Some(ext) = path.extension() {
            result.set_extension(format!("{}.{}", statusname, ext.to_string_lossy()));
        } else {
            result.set_extension(format!("{}", statusname));
        }
        result
    }

    pub fn path_reject(path: &Path) -> PathBuf {
        Self::path_with_status_extension(path, "reject")
    }

    pub fn path_keep(path: &Path) -> PathBuf {
        Self::path_with_status_extension(path, "keep")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process_keep(input: &str) -> String {
        let path1 = PathBuf::from(input);
        let path2 = PathUtil::path_keep(&path1);
        path2.to_string_lossy().to_string()
    }

    fn process_reject(input: &str) -> String {
        let path1 = PathBuf::from(input);
        let path2 = PathUtil::path_reject(&path1);
        path2.to_string_lossy().to_string()
    }

    #[test]
    fn test_10000_keep() {
        assert_eq!(process_keep("mine-event/20220710-054915-1251916462"), "mine-event/20220710-054915-1251916462.keep");
        assert_eq!(process_keep("mine-event/20220710-054915-1251916462.asm"), "mine-event/20220710-054915-1251916462.keep.asm");
        assert_eq!(process_keep("mine-event/20220710-054915-1251916462.keep.asm"), "mine-event/20220710-054915-1251916462.keep.keep.asm");
        assert_eq!(process_keep("mine-event/20220710-054915-1251916462.keep.keep.asm"), "mine-event/20220710-054915-1251916462.keep.keep.keep.asm");
        assert_eq!(process_keep("mine-event/20220710-054915-1251916462.a.b.asm"), "mine-event/20220710-054915-1251916462.a.b.keep.asm");
        assert_eq!(process_keep("readme.txt"), "readme.keep.txt");
    }

    #[test]
    fn test_10001_reject() {
        assert_eq!(process_reject("mine-event/20220710-054915-1251916462"), "mine-event/20220710-054915-1251916462.reject");
        assert_eq!(process_reject("mine-event/20220710-054915-1251916462.asm"), "mine-event/20220710-054915-1251916462.reject.asm");
        assert_eq!(process_reject("mine-event/20220710-054915-1251916462.keep.asm"), "mine-event/20220710-054915-1251916462.keep.reject.asm");
        assert_eq!(process_reject("mine-event/20220710-054915-1251916462.keep.keep.asm"), "mine-event/20220710-054915-1251916462.keep.keep.reject.asm");
        assert_eq!(process_reject("readme.txt"), "readme.reject.txt");
    }
}
