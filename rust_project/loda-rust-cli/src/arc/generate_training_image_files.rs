use super::{Color, Image, ImageSize};
use anyhow::Context;
use std::path::Path;

struct ImageExport;

impl ImageExport {
    fn save(image: &Image, path: &Path) -> anyhow::Result<()> {
        let size: ImageSize = image.size();
        if size.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let mut output = image_crate::ImageBuffer::new(size.width as u32, size.height as u32);
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let color_symbol: u8 = image.get(x as i32, y as i32).unwrap_or(255);
            let rgb: u32 = Color::rgb(color_symbol);
            let r: u8 = ((rgb >> 16) & 255) as u8;
            let g: u8 = ((rgb >> 8) & 255) as u8;
            let b: u8 = (rgb & 255) as u8;
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
    fn test_10000_create_arc_image_file() -> anyhow::Result<()> {
        // Arrange
        let tempdir = tempfile::tempdir().unwrap();
        let basedir = PathBuf::from(&tempdir.path()).join("test_10000_create_arc_image_file");
        fs::create_dir(&basedir)?;
        let path: PathBuf = basedir.join("output.png");

        let pixels: Vec<u8> = vec![
            0, 1, 2, 3,
            4, 5, 6, 7,
            8, 9, 10, 255,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        
        // Act
        ImageExport::save(&input, &path)?;

        // Assert
        assert_eq!(path.is_file(), true);
                
        let filesize: u64 = path.metadata()?.len();
        assert_eq!(filesize > 10, true);

        Ok(())
    }
}
