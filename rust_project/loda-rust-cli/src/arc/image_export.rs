use super::{Color, Image, ImageSize};
use anyhow::Context;
use std::path::Path;

/// Converts from a color symbol `[0..12]` to a normalized color value in the range `[0..255]`.
/// 
/// * The smallest value 0 is mapped to 0.
/// * The largest value 12 is mapped to 255.
/// * Values beyond 12 are mapped to 255.
fn normalize_color(color: u8) -> u8 {
    let color_normalized: u16 = (color as u16) * 255 / 12;
    if color_normalized > 255 {
        return 255;
    }
    color_normalized as u8
}

/// The vision transformer ViT code normalizes the color values to the range 0.0 to 1.0.
/// If some of the ARC tasks doesn't make use of the lower color values, then the ViT's normalization
/// will mess up the the lower color values.
/// 
/// It's better to pick two colors that are always present in the image, and use those for min and max.
/// so that the ViT's normalization will not mess up the color values.
fn remap_color(color: u8) -> u8 {
    match color {
        0 => 1, // ARC color. Not all tasks use this color. So it's ill-suited for the minimum value.
        1 => 2, // ARC color. Not all tasks use this color.
        2 => 3, // ARC color. Not all tasks use this color.
        3 => 4, // ARC color. Not all tasks use this color.
        4 => 5, // ARC color. Not all tasks use this color.
        5 => 6, // ARC color. Not all tasks use this color.
        6 => 7, // ARC color. Not all tasks use this color.
        7 => 8, // ARC color. Not all tasks use this color.
        8 => 9, // ARC color. Not all tasks use this color.
        9 => 10, // ARC color. Not all tasks use this color.
        10 => 11, // For images smaller than 30x30 this color is always present. But for 30x30 it's not present.
        11 => 0, // Always present in the image, the padding around the image.
        12 => 12, // Always present in the image, the highlight color.
        _ => 11, // There shouldn't be colors outside the range 0..12, but just in case.
    }
}

pub trait ImageExport {
    /// Save the image to a file.
    /// 
    /// The `path` must be absolute.
    /// 
    /// Supported file types (lossless): `png`. Recommended.
    /// 
    /// Supported file types (lossy): `jpg`. Not recommended.
    /// 
    /// Unsupported file types: `webp`.
    fn save_as_file(&self, path: &Path) -> anyhow::Result<()>;

    fn save_as_file_onechannel_raw(&self, path: &Path) -> anyhow::Result<()>;
    fn save_as_file_onechannel_normalized(&self, path: &Path) -> anyhow::Result<()>;
    fn save_as_file_onechannel_remap_and_normalized(&self, path: &Path) -> anyhow::Result<()>;
}

impl ImageExport for Image {
    fn save_as_file(&self, path: &Path) -> anyhow::Result<()> {
        let size: ImageSize = self.size();
        if size.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let mut output = image_crate::ImageBuffer::new(size.width as u32, size.height as u32);
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let color_symbol: u8 = self.get(x as i32, y as i32).unwrap_or(255);
            let rgb: u32 = Color::rgb(color_symbol);
            let r: u8 = ((rgb >> 16) & 255) as u8;
            let g: u8 = ((rgb >> 8) & 255) as u8;
            let b: u8 = (rgb & 255) as u8;
            *pixel = image_crate::Rgb([r, g, b]);
        }
        output.save(path).context("output.save")?;
        Ok(())
    }

    fn save_as_file_onechannel_raw(&self, path: &Path) -> anyhow::Result<()> {
        let size: ImageSize = self.size();
        if size.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let mut output = image_crate::ImageBuffer::new(size.width as u32, size.height as u32);
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let color_symbol: u8 = self.get(x as i32, y as i32).unwrap_or(255);
            let r: u8 = color_symbol;
            let g: u8 = 0;
            let b: u8 = 0;
            *pixel = image_crate::Rgb([r, g, b]);
        }
        output.save(path).context("output.save")?;
        Ok(())
    }

    fn save_as_file_onechannel_normalized(&self, path: &Path) -> anyhow::Result<()> {
        let size: ImageSize = self.size();
        if size.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let mut output = image_crate::ImageBuffer::new(size.width as u32, size.height as u32);
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let color_symbol: u8 = self.get(x as i32, y as i32).unwrap_or(255);
            let r: u8 = normalize_color(color_symbol);
            let g: u8 = 0;
            let b: u8 = 0;
            *pixel = image_crate::Rgb([r, g, b]);
        }
        output.save(path).context("output.save")?;
        Ok(())
    }

    fn save_as_file_onechannel_remap_and_normalized(&self, path: &Path) -> anyhow::Result<()> {
        let size: ImageSize = self.size();
        if size.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let mut output = image_crate::ImageBuffer::new(size.width as u32, size.height as u32);
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let color_symbol: u8 = self.get(x as i32, y as i32).unwrap_or(255);
            let r: u8 = normalize_color(remap_color(color_symbol));
            let g: u8 = 0;
            let b: u8 = 0;
            *pixel = image_crate::Rgb([r, g, b]);
        }
        output.save(path).context("output.save")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_normalize_color() {
        let colors = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
        ];
        let mut normalized: Vec<u8> = Vec::new();
        for color in colors {
            normalized.push(normalize_color(color));
        }
        assert_eq!(normalized, vec![
            0, 21, 42, 63, 85, 106, 127, 148, 170, 191, 212, 233, 255, 255, 255,
        ]);
    }

    #[test]
    fn test_20000_remap_color() {
        let colors = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
        ];
        let mut normalized: Vec<u8> = Vec::new();
        for color in colors {
            normalized.push(remap_color(color));
        }
        assert_eq!(normalized, vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 12, 11, 11,
        ]);
    }

    #[test]
    fn test_30000_create_arc_image_file() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_30000_create_arc_image_file");
        fs::create_dir(&basedir)?;
        let path: PathBuf = basedir.join("output.png");

        let pixels: Vec<u8> = vec![
            0, 1, 2, 3,
            4, 5, 6, 7,
            8, 9, 10, 255,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        
        // Act
        input.save_as_file(&path)?;

        // Assert
        assert_eq!(path.is_file(), true);
                
        let filesize: u64 = path.metadata()?.len();
        assert_eq!(filesize > 10, true);

        Ok(())
    }
}
