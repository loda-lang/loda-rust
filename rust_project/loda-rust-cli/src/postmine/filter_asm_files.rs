use std::path::PathBuf;

/// We are only interested in `.asm` files 
/// that lives inside the `loda-programs` repo's `oeis` dir.
/// 
/// We don't care about `*.md`, `*.txt`, `*.png` files.
/// So if there is a change to the `README.md` then it doesn't get through this filter.
pub fn filter_asm_files(paths1: &Vec<PathBuf>) -> Vec<PathBuf> {
    let paths2: Vec<&PathBuf> = paths1.iter().filter(|path| 
        path.is_file() && path.is_extension_asm() && path.is_containing_oeis()
    ).collect();
    paths2.iter().map(|path| PathBuf::from(path)).collect()
}

trait FilterPathExtension {
    fn is_extension_asm(&self) -> bool;
    fn is_containing_oeis(&self) -> bool;
}

impl FilterPathExtension for PathBuf {
    fn is_extension_asm(&self) -> bool {
        if let Some(ext) = self.extension() {
            if ext.to_string_lossy() == "asm" {
                return true;
            }
        }
        false    
    }

    /// Rough sanity check - are we really inside the `loda-programs` repo's `oeis` dir
    fn is_containing_oeis(&self) -> bool {
        let path_string: String = self.to_string_lossy().to_string();
        if path_string.contains("oeis") {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_extension_asm(input: &str) -> bool {
        let path = PathBuf::from(input);
        path.is_extension_asm()
    }

    fn is_containing_oeis(input: &str) -> bool {
        let path = PathBuf::from(input);
        path.is_containing_oeis()
    }

    #[test]
    fn test_10000_is_extension_asm() {
        assert_eq!(is_extension_asm("mine-event/20220710-054915-1251916462"), false);
        assert_eq!(is_extension_asm("mine-event/20220710-054915-1251916462.asm"), true);
        assert_eq!(is_extension_asm("mine-event/20220710-054915-1251916462.keep.asm"), true);
        assert_eq!(is_extension_asm("A001014.asm"), true);
        assert_eq!(is_extension_asm("A001014.asm.bak"), false);
        assert_eq!(is_extension_asm(""), false);
        assert_eq!(is_extension_asm("."), false);
        assert_eq!(is_extension_asm(".."), false);
    }

    #[test]
    fn test_20000_is_containing_oeis() {
        assert_eq!(is_containing_oeis("mine-event/20220710-054915-1251916462"), false);
        assert_eq!(is_containing_oeis("A001014.asm"), false);
        assert_eq!(is_containing_oeis("oeis/001/A001014.asm"), true);
        assert_eq!(is_containing_oeis("oeis/001"), true);
        assert_eq!(is_containing_oeis("oeis/full_check.txt"), true);
        assert_eq!(is_containing_oeis(""), false);
        assert_eq!(is_containing_oeis("."), false);
        assert_eq!(is_containing_oeis(".."), false);
    }
}
