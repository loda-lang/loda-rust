use super::{ImageFill, ConnectedComponent, PixelConnectivity, ImageOverlay, ImageObjectEnumerate};
use super::{Histogram, Image, ImageHistogram, ImageMask, Rectangle, ImageMix, ImageSize, MixMode, ImageMaskCount, ImageCrop};

/// A rectangle filled with a single solid color and no other colors are present inside the object.
#[derive(Clone, Debug)]
pub struct SingleColorObjectRectangle {
    pub color: u8,
    pub mask: Image,
    pub bounding_box: Rectangle,
    pub mass: u16,
    pub is_square: bool,
}

#[derive(Clone, Debug)]
pub struct SingleColorObjectClusterContainer {
    pub cluster_vec: Vec::<SingleColorObjectCluster>,
    pub enumerated_clusters: Image,
}

/// A mask of pixels that have the same color, but isn't fully connected.
/// 
/// It may be separate objects.
/// 
/// It may be a diagonal line that has one color, but the pixels that aren't on the diagonal
/// having a different color.
/// 
/// The rectangle that contains the object also contains 1 or more pixels of different colors.
#[derive(Clone, Debug)]
pub struct SingleColorObjectSparse {
    pub color: u8,
    pub mask: Image,

    /// Bounding box of the mask
    pub bounding_box: Rectangle,

    /// Number of pixels with same value as `color`
    pub mass_object: u16,

    /// Number of pixels different than `color`
    pub mass_non_object: u16,

    /// Histogram of the non-object pixels 
    pub histogram_non_object: Histogram,

    /// Child objects by analyzing with `PixelConnectivity4`
    pub container4: Option<SingleColorObjectClusterContainer>,

    /// Child objects by analyzing with `PixelConnectivity8`
    pub container8: Option<SingleColorObjectClusterContainer>,

    // Future experiments:
    // are container4 all single pixels?
    // Noise color for single pixel noise
    // Are container4 and container8 identical? same shape, same number of holes.
    // histogram of areas between clusters.
    // number of holes
    // are the non-object pixels a single color
    // If there is only a single color that isn't ObjectWithOneColor
    // then it may be because it's the background color.
    // compare the background color across all the single objects if it's the same.
    // child objects
    // surrounding objects
    // Detect objects with multiple colors
}

impl SingleColorObjectSparse {
    fn create(color: u8, image: &Image, mask: Image, rect: Rectangle) -> anyhow::Result<Self> {
        let cropped_object: Image = image.crop(rect)?;
        let mut histogram: Histogram = cropped_object.histogram_all();
        let mass_object: u16 = histogram.get(color).min(u16::MAX as u32) as u16;
        histogram.set_counter_to_zero(color);
        let mass_non_object: u16 = histogram.sum().min(u16::MAX as u32) as u16;
        let mut instance = SingleColorObjectSparse {
            color,
            mask,
            bounding_box: rect,
            mass_object,
            mass_non_object,
            histogram_non_object: histogram,
            container4: None,
            container8: None,
        };
        instance.analyze(PixelConnectivity::Connectivity4)?;
        instance.analyze(PixelConnectivity::Connectivity8)?;
        Ok(instance)
    }

    /// The `connectivity` parameter is for choosing between 4-connected and 8-connected.
    fn analyze(&mut self, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        // Objects that is not the background
        let cropped_mask: Image = self.mask.crop(self.bounding_box)?;
        let ignore_mask: Image = cropped_mask.invert_mask();

        let blank = Image::zero(cropped_mask.width(), cropped_mask.height());
        let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(connectivity, &blank, &ignore_mask)?;

        let mut objects_with_hole_vec = Vec::<Image>::new();
        let mut cluster_mask = Image::zero(cropped_mask.width(), cropped_mask.height());
        for object in &object_mask_vec {
            // println!("object: {:?}", object);
            let rect: Rectangle = match object.bounding_box() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let cropped_object: Image = object.crop(rect)?;
            // println!("cropped_object: {:?}", cropped_object);
            let mut object_image: Image = cropped_object.clone();

            // flood fill at every border pixel around the object
            let x1: i32 = (object_image.width() as i32) - 1;
            let y1: i32 = (object_image.height() as i32) - 1;
            for y in 0..(object_image.height() as i32) {
                for x in 0..(object_image.width() as i32) {
                    if x > 0 && x < x1 && y > 0 && y < y1 { 
                        continue;
                    }
                    let pixel: u8 = object_image.get(x, y).unwrap_or(255);
                    if pixel == 0 {
                        match connectivity {
                            PixelConnectivity::Connectivity4 => {
                                object_image.flood_fill4(x, y, 0, 1);
                            },
                            PixelConnectivity::Connectivity8 => {
                                object_image.flood_fill8(x, y, 0, 1);
                            },
                        }
                    }
                }
            }
            // println!("object_image: {:?}", object_image);

            // if there are unfilled areas, then it's because there is one or more holes
            let count: u16 = object_image.mask_count_zero();
            if count > 0 {
                // println!("found hole with count={}", count);
                objects_with_hole_vec.push(object.clone());
            }

            // fill out the holes
            let inverted_mask: Image = object_image.invert_mask();
            // println!("inverted_mask: {:?}", inverted_mask);
            let combined: Image = cropped_object.mix(&inverted_mask, MixMode::BooleanOr)?;

            cluster_mask = cluster_mask.overlay_with_mask_and_position(&combined, &combined, rect.x() as i32, rect.y() as i32)?;
        }

        // Find the clusters
        let ignore_mask: Image = cluster_mask.invert_mask();
        let cluster_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(connectivity, &blank, &ignore_mask)?;

        let enumerated_clusters: Image = Image::object_enumerate(&cluster_mask_vec)?;

        // println!("number of clusters: {}", object_mask_vec2.len());
        let mut cluster_vec = Vec::<SingleColorObjectCluster>::new();
        for (index, cluster_mask) in cluster_mask_vec.iter().enumerate() {
            let item = SingleColorObjectCluster {
                cluster_id: index + 1,
                mask: cluster_mask.clone(),
                one_or_more_holes: false,
            };
            cluster_vec.push(item);
        }

        // Compare what cluster an "object-with-hole" belongs to, by looking at the enumerated_cluster
        // if there is overlap, then it belongs to that cluster.
        // then flag that cluster as having one or more holes.
        for object in &objects_with_hole_vec {
            let h: Histogram = enumerated_clusters.histogram_with_mask(&object)?;
            let cluster_id: u8 = match h.most_popular_color_disallow_ambiguous() {
                Some(value) => value,
                None => {
                    // println!("color: {} ambiguous what cluster the object belong to", self.color);
                    continue;
                }
            };
            // println!("color: {} connectivity: {:?} cluster_id: {}", self.color, connectivity, cluster_id);
            if cluster_id == 0 {
                // println!("cluster_id is not supposed to be 0. Ignoring this object.");
                continue;
            }
            let index = (cluster_id - 1) as usize;
            if index >= cluster_vec.len() {
                // println!("cluster_id is out of range. Ignoring this object.");
                continue;
            }
            cluster_vec[index].one_or_more_holes = true;

        }

        let container = SingleColorObjectClusterContainer {
            cluster_vec,
            enumerated_clusters,
        };

        match connectivity {
            PixelConnectivity::Connectivity4 => {
                self.container4 = Some(container);
            },
            PixelConnectivity::Connectivity8 => {
                self.container8 = Some(container);
            },
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SingleColorObjectCluster {
    pub cluster_id: usize,
    pub mask: Image,
    pub one_or_more_holes: bool,

    // Future experiments:
    // mass_cluster,
    // mass_holes,
    // is a box
    // shape type: L shape, T shape, + shape, diagonal shape, other shape
    // symmetry
    // outermost pixels have same color
    // histogram of all holes in this cluster.
    // number of holes in this cluster
    // list of holes in this cluster
    // shape of each hole. square, non-square, rectangular, other.
    // color of each hole. same, different.
    // shape of cluster
}

#[derive(Clone, Debug)]
pub struct SingleColorObjects {
    pub image_size: ImageSize,
    pub rectangle_vec: Vec<SingleColorObjectRectangle>,
    pub sparse_vec: Vec<SingleColorObjectSparse>,
}

impl SingleColorObjects {
    pub fn find_objects(image: &Image) -> anyhow::Result<Self> {
        if image.is_empty() {
            return Err(anyhow::anyhow!("The image must be 1x1 or bigger"));
        }
        let image_histogram: Histogram = image.histogram_all();
        let mut rectangle_vec = Vec::<SingleColorObjectRectangle>::new();
        let mut sparse_vec = Vec::<SingleColorObjectSparse>::new();
        for (count, color) in image_histogram.pairs_ordered_by_color() {
            let mask: Image = image.to_mask_where_color_is(color);
            let rect: Rectangle = match mask.bounding_box() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let mass: u16 = (rect.width() as u16) * (rect.height() as u16);
            if count != (mass as u32) {
                let item: SingleColorObjectSparse = SingleColorObjectSparse::create(color, image, mask, rect)?;
                sparse_vec.push(item);
                continue;
            }

            let is_square: bool = rect.width() == rect.height();
            let item = SingleColorObjectRectangle {
                color,
                mask,
                bounding_box: rect,
                mass,
                is_square,
            };
            rectangle_vec.push(item);
        }
        let instance = Self {
            image_size: image.size(),
            rectangle_vec,
            sparse_vec,
        };
        instance.verify_all_pixels_are_accounted_for()?;
        Ok(instance)
    }

    /// Verify that the every pixel in the image are fully explained by the 
    /// `rectangle_vec` and the `sparse_vec`. If one or more pixels isn't accounted for,
    /// then something must have gone wrong while analyzing the pixels.
    fn verify_all_pixels_are_accounted_for(&self) -> anyhow::Result<()> {
        let mut result_mask = Image::zero(self.image_size.width, self.image_size.height);
        for object in &self.rectangle_vec {
            result_mask = result_mask.mix(&object.mask, MixMode::Plus)?;
        }
        for object in &self.sparse_vec {
            result_mask = result_mask.mix(&object.mask, MixMode::Plus)?;
        }
        let actual_mass: u16 = result_mask.mask_count_one();
        let expected_mass: u16 = (self.image_size.width as u16) * (self.image_size.height as u16);
        if actual_mass != expected_mass {
            return Err(anyhow::anyhow!("The objects doesn't cover the image correctly. Each pixel is supposed to be counted once, but was either not counted at all, or counted multiple times. Cannot explain all the pixels in the image."));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_object_rectangle() {
        // Arrange
        let pixels: Vec<u8> = vec![
            1, 2, 3,
            4, 5, 6,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");

        // Assert
        assert_eq!(actual.rectangle_vec.len(), 6);
        assert_eq!(actual.sparse_vec.len(), 0);
    }

    #[test]
    fn test_20000_object_sparse() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 6,
            6, 3, 3,
        ];
        let input: Image = Image::try_create(3, 2, pixels).expect("image");

        // Act
        let actual: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");

        // Assert
        assert_eq!(actual.rectangle_vec.len(), 2);
        assert_eq!(actual.sparse_vec.len(), 1);

        let object: &SingleColorObjectSparse = actual.sparse_vec.first().expect("1 instance");
        assert_eq!(object.bounding_box, Rectangle::new(0, 0, 3, 2));

        let expected_pixels: Vec<u8> = vec![
            0, 0, 1,
            1, 0, 0,
        ];
        let expected_mask: Image = Image::try_create(3, 2, expected_pixels).expect("image");
        assert_eq!(object.mask, expected_mask);

        assert_eq!(object.mass_object, 2);
        assert_eq!(object.mass_non_object, 4);

        {
            let mut histogram = Histogram::new();
            histogram.increment(5);
            histogram.increment(5);
            histogram.increment(3);
            histogram.increment(3);
            assert_eq!(object.histogram_non_object, histogram);
        }
    }

    #[test]
    fn test_30000_cluster() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 7, 7, 7, 7, 0, 0, 0,
            0, 0, 7, 0, 7, 0, 0, 0, 0,
            0, 0, 7, 7, 7, 0, 7, 0, 0,
            0, 0, 7, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 0, 0, 7, 7, 0, 0,
            0, 0, 0, 7, 7, 7, 7, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(9, 8, pixels).expect("image");
        
        // Act
        let actual: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");

        // Assert
        assert_eq!(actual.rectangle_vec.len(), 0);
        assert_eq!(actual.sparse_vec.len(), 2);
        {
            let object: &SingleColorObjectSparse = &actual.sparse_vec[0];
            assert_eq!(object.color, 0);
            assert_eq!(object.bounding_box, Rectangle::new(0, 0, 9, 8));
        }
        {
            let object: &SingleColorObjectSparse = &actual.sparse_vec[1];
            assert_eq!(object.color, 7);
            assert_eq!(object.bounding_box, Rectangle::new(2, 1, 5, 6));
            assert_eq!(object.mass_object, 18);
        }

        let object: &SingleColorObjectSparse = &actual.sparse_vec[1];
        let container: &SingleColorObjectClusterContainer = object.container4.as_ref().expect("container");
        assert_eq!(container.cluster_vec.len(), 2);

        {
            let expected_pixels: Vec<u8> = vec![
                1, 1, 1, 1, 0,
                1, 1, 1, 0, 0,
                1, 1, 1, 0, 2,
                1, 0, 0, 0, 2,
                0, 0, 0, 2, 2,
                0, 2, 2, 2, 2,
            ];
            let expected: Image = Image::try_create(5, 6, expected_pixels).expect("image");
            assert_eq!(container.enumerated_clusters, expected);
        }

        {
            let cluster: &SingleColorObjectCluster = &container.cluster_vec[0];
            assert_eq!(cluster.cluster_id, 1);
            assert_eq!(cluster.one_or_more_holes, true);
        }
        {
            let cluster: &SingleColorObjectCluster = &container.cluster_vec[1];
            assert_eq!(cluster.cluster_id, 2);
            assert_eq!(cluster.one_or_more_holes, false);
        }
    }
}
