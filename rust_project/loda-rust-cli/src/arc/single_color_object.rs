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

    /// This image has the same size as the `bounding_box`.
    pub enumerated_clusters_cropped: Image,

    /// This image has the same size as the original image.
    pub enumerated_clusters_uncropped: Image,

    /// This image has the same size as the `bounding_box`.
    pub holes_mask_cropped: Image,

    /// This image has the same size as the original image.
    pub holes_mask_uncropped: Image,
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

    /// This image has the same size as the original image.
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

    /// When `true` then `PixelConnectivity4` and `PixelConnectivity8` yields the same child objects, 
    /// and things are easy.
    /// 
    /// When `false` then `PixelConnectivity4` and `PixelConnectivity8` yields different masks,
    /// and extra care is needed to determine wheter to use `PixelConnectivity4` or `PixelConnectivity8`.
    pub connectivity48_identical: bool,

    // Future experiments:
    // are container4 all single pixels?
    // Noise color for single pixel noise
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
        let verbose = false;
        if verbose {
            println!("SingleColorObjectSparse.create: color: {} mask: {:?} rect: {:?}", color, mask, rect);
        }
        let cropped_object: Image = image.crop(rect)?;
        let mut histogram: Histogram = cropped_object.histogram_all();
        let mass_object: u16 = histogram.get(color).min(u16::MAX as u32) as u16;
        histogram.set_counter_to_zero(color);
        let mass_non_object: u16 = histogram.sum().min(u16::MAX as u32) as u16;
        if verbose {
            println!("mass_object: {} mass_non_object: {}", mass_object, mass_non_object);
        }
        let mut instance = SingleColorObjectSparse {
            color,
            mask,
            bounding_box: rect,
            mass_object,
            mass_non_object,
            histogram_non_object: histogram,
            container4: None,
            container8: None,
            connectivity48_identical: false,
        };
        match instance.analyze(PixelConnectivity::Connectivity4) {
            Ok(()) => {},
            Err(error) => {
                if verbose {
                    println!("SingleColorObjectSparse.create: analyze with Connectivity4 error: {:?}", error);
                }
            }
        }
        match instance.analyze(PixelConnectivity::Connectivity8) {
            Ok(()) => {},
            Err(error) => {
                if verbose {
                    println!("SingleColorObjectSparse.create: analyze with Connectivity8 error: {:?}", error);
                }
            }
        }
        instance.update_connectivity48_identical()?;
        Ok(instance)
    }

    /// Detect identical masks for `PixelConnectivity4` and `PixelConnectivity8`.
    fn update_connectivity48_identical(&mut self) -> anyhow::Result<()> {
        let container4: &SingleColorObjectClusterContainer = match &self.container4 {
            Some(value) => value,
            None => {
                return Ok(());
            }
        }; 
        let container8: &SingleColorObjectClusterContainer = match &self.container8 {
            Some(value) => value,
            None => {
                return Ok(());
            }
        };
        if container4.cluster_vec.len() != container8.cluster_vec.len() {
            return Ok(());
        }
        for (cluster4, cluster8) in container4.cluster_vec.iter().zip(container8.cluster_vec.iter()) {
            if cluster4.mask != cluster8.mask {
                return Ok(());
            }
        }
        self.connectivity48_identical = true;
        Ok(())
    }

    /// The `connectivity` parameter is for choosing between 4-connected and 8-connected.
    fn analyze(&mut self, connectivity: PixelConnectivity) -> anyhow::Result<()> {
        // Objects that is not the background
        let cropped_mask: Image = self.mask.crop(self.bounding_box)?;
        let ignore_mask: Image = cropped_mask.invert_mask();

        let blank = Image::zero(cropped_mask.width(), cropped_mask.height());
        let object_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(connectivity, &blank, &ignore_mask)?;

        // println!("analyze: color: {} object_mask_vec.len(): {}", self.color, object_mask_vec.len());

        let mut accumulated_holes_mask: Image = Image::zero(cropped_mask.width(), cropped_mask.height());
        let mut cluster_vec = Vec::<SingleColorObjectCluster>::new();
        for (index, object) in object_mask_vec.iter().enumerate() {
            let rect: Rectangle = match object.bounding_box() {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let cropped_object: Image = object.crop(rect)?;
            let mut object_image: Image = cropped_object.clone();

            // flood fill at every border pixel around the object
            object_image.border_flood_fill(0, 1, connectivity);

            // if there are unfilled areas, then it's because there is one or more holes
            let count: u16 = object_image.mask_count_zero();
            let one_or_more_holes: bool = count > 0;
            let mut number_of_holes: u16 = 0;
            if one_or_more_holes {
                // println!("found hole with count={}", count);

                let hole_mask_vec: Vec<Image> = ConnectedComponent::find_objects_with_ignore_mask(connectivity, &object_image, &object_image)?;
                // println!("color: {} connectivity: {:?} number of holes: {}", self.color, connectivity, hole_mask_vec.len());
                number_of_holes = hole_mask_vec.len().min(u16::MAX as usize) as u16;

                // Draw the hole into the accumulated holes mask
                let inverted_mask: Image = object_image.invert_mask();
                let mut hole_mask = Image::zero(cropped_mask.width(), cropped_mask.height());
                hole_mask = hole_mask.overlay_with_position(&inverted_mask, rect.min_x(), rect.min_y())?;
                accumulated_holes_mask = accumulated_holes_mask.mix(&hole_mask, MixMode::BooleanOr)?;
            }

            let mass_cluster: u16 = object.mask_count_one();
            let item = SingleColorObjectCluster {
                cluster_id: index + 1,
                mask: object.clone(),
                one_or_more_holes,
                mass_cluster,
                number_of_holes,
            };
            cluster_vec.push(item);
        }

        // The holes
        let mut holes_mask_uncropped: Image = Image::zero(self.mask.width(), self.mask.height());
        holes_mask_uncropped = holes_mask_uncropped.overlay_with_position(&accumulated_holes_mask, self.bounding_box.min_x(), self.bounding_box.min_y())?;

        // Enumerate the clusters
        let enumerated_clusters: Image = Image::object_enumerate(&object_mask_vec)?;
        let mut enumerated_clusters_uncropped: Image = Image::zero(self.mask.width(), self.mask.height());
        enumerated_clusters_uncropped = enumerated_clusters_uncropped.overlay_with_position(&enumerated_clusters, self.bounding_box.min_x(), self.bounding_box.min_y())?;

        let container = SingleColorObjectClusterContainer {
            cluster_vec,
            enumerated_clusters_cropped: enumerated_clusters,
            enumerated_clusters_uncropped,
            holes_mask_cropped: accumulated_holes_mask,
            holes_mask_uncropped,
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
    pub mass_cluster: u16,
    pub number_of_holes: u16,

    // Future experiments:
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
            // println!("find_objects color: {} mask: {:?}", color, mask);
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

    /// Determine if there are there sprinkled noise pixels over the image.
    /// 
    /// The noise pixels are isolated lonely pixels with a mass of 1 pixel.
    /// 
    /// Pick the color with the most noise pixels.
    /// 
    /// Returns `None` when it's ambiguous what is the most popular noise color.
    /// 
    /// Returns `None` when the all colors have an area with `mass > 1`. And thus no single pixels can be identified.
    pub fn single_pixel_noise_color(&self) -> Option<u8> {
        let mut histogram: Histogram = Histogram::new();
        for object in &self.rectangle_vec {
            if object.mass == 1 {
                histogram.increment(object.color);
            }
        }
        for object in &self.sparse_vec {
            let container4: &SingleColorObjectClusterContainer = match &object.container4 {
                Some(value) => value,
                None => {
                    continue;
                }
            }; 
            if container4.cluster_vec.len() != object.mass_object as usize {
                // there are fewer clusters than the mass of the object.
                // one ore more of the clusters are bigger than 2 pixels.
                continue;
            }
            let container8: &SingleColorObjectClusterContainer = match &object.container8 {
                Some(value) => value,
                None => {
                    continue;
                }
            };
            let mut mass_of_clusters_bigger_than_1pixel: usize = 0;
            for cluster in &container8.cluster_vec {
                let cluster_mass: u16 = cluster.mask.mask_count_one();
                if cluster_mass >= 2 {
                    // the cluster is bigger than 1 pixel.
                    mass_of_clusters_bigger_than_1pixel += cluster_mass as usize;
                }
            }

            // A low value rejects diagonal lines.
            // A high value allows for more diagonal lines.
            let fuzzy_percent: usize = 20;
            
            if mass_of_clusters_bigger_than_1pixel > ((object.mass_object as usize) * fuzzy_percent / 100) {
                // Ignoring this cluster. It seems to contain diagonal lines, so it's not noise.
                // with pixel-connectivity-4 there are as many clusters as there are pixels with the object color.
                // however when using pixel-connectivity-8, there are may be diagonal connected pixels. 
                // If all of the pixels are in a diagonal line, then it's not noise.
                // Thus only reject when exceeding 20% of pixels are allowed to be greater than 1 pixel.
                // println!("ignoring this cluster mass_of_clusters_bigger_than_1pixel: {}", mass_of_clusters_bigger_than_1pixel);
                continue;
            }
            // There are as many clusters as there are pixels with the object color.
            // the clusters are separated by 1 or more pixels, so there is a high chance that it's noise.
            histogram.increment_by(object.color, object.mass_object as u32);
        }
        histogram.most_popular_color_disallow_ambiguous()
    }

    /// Extracts the `mass` from all objects. Clamp the `mass` to a maximum of 255.
    /// 
    /// In every pixel of the image, go through all the objects and save the mass as the pixel color.
    /// 
    /// Returns an image with the same size as the input image.
    pub fn mass_as_image(&self, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        let mut result_image = Image::zero(self.image_size.width, self.image_size.height);
        for object in &self.rectangle_vec {
            // println!("rectangle: {:?}", object);
            let color: u8 = object.mass.min(255) as u8;
            // println!("mask: {:?} mass: {}", object.mask, color);
            result_image = object.mask.select_from_image_and_color(&result_image, color)?;
        }
        for object in &self.sparse_vec {
            // println!("sparse: {:?}", object);
            let optional_container: Option<&SingleColorObjectClusterContainer> = match connectivity {
                PixelConnectivity::Connectivity4 => object.container4.as_ref(),
                PixelConnectivity::Connectivity8 => object.container8.as_ref(),
            };
            let container: &SingleColorObjectClusterContainer = match optional_container {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Missing container"));
                }
            };
            // println!("enumerated_clusters_cropped: {:?}", container.enumerated_clusters_cropped);
            // println!("enumerated_clusters_uncropped: {:?}", container.enumerated_clusters_uncropped);
            for cluster in &container.cluster_vec {
                // println!("cluster: {:?}", cluster);
                let color: u8 = cluster.mass_cluster.min(255) as u8;
                let mut mask: Image = Image::zero(self.image_size.width, self.image_size.height);
                mask = mask.overlay_with_position(&cluster.mask, object.bounding_box.min_x(), object.bounding_box.min_y())?;
                // println!("mask: {:?} mass: {}", cluster.mask, color);
                result_image = mask.select_from_image_and_color(&result_image, color)?;
            }
        }
        Ok(result_image)
    }

    /// Extracts the hole masks from objects with the specified `color`.
    /// 
    /// Returns an image with the same size as the input image.
    #[allow(dead_code)]
    pub fn holes_mask(&self, color: u8, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        for object in &self.sparse_vec {
            if object.color != color {
                continue;
            }
            let optional_container: Option<&SingleColorObjectClusterContainer> = match connectivity {
                PixelConnectivity::Connectivity4 => object.container4.as_ref(),
                PixelConnectivity::Connectivity8 => object.container8.as_ref(),
            };
            let container: &SingleColorObjectClusterContainer = match optional_container {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Missing container"));
                }
            };
            let result_image: Image = container.holes_mask_uncropped.clone();
            return Ok(result_image);
        }
        Err(anyhow::anyhow!("Color not found"))
    }

    /// Extracts the number of holes for each objects with the specified `color`.
    /// 
    /// Returns an image with the same size as the input image.
    pub fn holecount_image(&self, color: u8, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        for object in &self.sparse_vec {
            if object.color != color {
                continue;
            }
            let optional_container: Option<&SingleColorObjectClusterContainer> = match connectivity {
                PixelConnectivity::Connectivity4 => object.container4.as_ref(),
                PixelConnectivity::Connectivity8 => object.container8.as_ref(),
            };
            let container: &SingleColorObjectClusterContainer = match optional_container {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Missing container"));
                }
            };
            let mut accumulated_holes: Image = Image::zero(object.bounding_box.width(), object.bounding_box.height());
            for cluster in &container.cluster_vec {
                let number_of_holes: u8 = cluster.number_of_holes.min(255) as u8;
                accumulated_holes = cluster.mask.select_from_image_and_color(&accumulated_holes, number_of_holes)?;
            }
            let mut result_image: Image = Image::zero(self.image_size.width, self.image_size.height);
            result_image = result_image.overlay_with_position(&accumulated_holes, object.bounding_box.min_x(), object.bounding_box.min_y())?;
            return Ok(result_image);
        }
        Err(anyhow::anyhow!("Color not found"))
    }

    /// Mask of the objects with the specified `color`, where the holes have been filled in.
    /// 
    /// Returns an image with the same size as the input image.
    /// 
    /// Future experiment:
    /// Blur mask, grow it by 1 px in all directions, then subtract the original mask, to get the outline.
    #[allow(dead_code)]
    pub fn filled_holes_mask(&self, color: u8, connectivity: PixelConnectivity) -> anyhow::Result<Image> {
        for object in &self.rectangle_vec {
            if object.color != color {
                continue;
            }
            let mask: Image = object.mask.clone();
            return Ok(mask);
        }
        for object in &self.sparse_vec {
            if object.color != color {
                continue;
            }
            let optional_container: Option<&SingleColorObjectClusterContainer> = match connectivity {
                PixelConnectivity::Connectivity4 => object.container4.as_ref(),
                PixelConnectivity::Connectivity8 => object.container8.as_ref(),
            };
            let container: &SingleColorObjectClusterContainer = match optional_container {
                Some(value) => value,
                None => {
                    return Err(anyhow::anyhow!("Missing container"));
                }
            };
            let clusters: &Image = &container.enumerated_clusters_uncropped;
            let holes: &Image = &container.holes_mask_uncropped;
            let result_image: Image = clusters.mix(holes, MixMode::BooleanOr)?;
            return Ok(result_image);
        }
        Err(anyhow::anyhow!("Color not found"))
    }

    /// Check if the coordinate is on the corner of the bounding box.
    #[allow(dead_code)]
    pub fn corner_classification(&self, color: u8, x: i32, y: i32) -> u8 {
        for object in &self.rectangle_vec {
            if object.color != color {
                continue;
            }
            return object.bounding_box.corner_classification(x, y);
        }
        for object in &self.sparse_vec {
            if object.color != color {
                continue;
            }
            return object.bounding_box.corner_classification(x, y);
        }
        0
    }

    /// Check if the coordinate is inside the bounding box.
    #[allow(dead_code)]
    pub fn is_inside_bounding_box(&self, color: u8, x: i32, y: i32) -> bool {
        for object in &self.rectangle_vec {
            if object.color != color {
                continue;
            }
            return object.bounding_box.is_inside(x, y);
        }
        for object in &self.sparse_vec {
            if object.color != color {
                continue;
            }
            return object.bounding_box.is_inside(x, y);
        }
        false
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
                1, 0, 1, 0, 0,
                1, 1, 1, 0, 2,
                1, 0, 0, 0, 2,
                0, 0, 0, 2, 2,
                0, 2, 2, 2, 2,
            ];
            let expected: Image = Image::try_create(5, 6, expected_pixels).expect("image");
            assert_eq!(container.enumerated_clusters_cropped, expected);
        }

        {
            let expected_pixels: Vec<u8> = vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 1, 1, 1, 1, 0, 0, 0,
                0, 0, 1, 0, 1, 0, 0, 0, 0,
                0, 0, 1, 1, 1, 0, 2, 0, 0,
                0, 0, 1, 0, 0, 0, 2, 0, 0,
                0, 0, 0, 0, 0, 2, 2, 0, 0,
                0, 0, 0, 2, 2, 2, 2, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ];
            let expected: Image = Image::try_create(9, 8, expected_pixels).expect("image");
            assert_eq!(container.enumerated_clusters_uncropped, expected);
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

    #[test]
    fn test_40000_single_pixel_noise_color_from_rectangle_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 0, 5, 5, 5, 5,
            7, 7, 7, 0, 0, 5, 5, 5, 5,
            7, 7, 7, 0, 0, 5, 5, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            9, 8, 8, 8, 8, 8, 8, 5, 5,
        ];
        let input: Image = Image::try_create(9, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Option<u8> = objects.single_pixel_noise_color();

        // Assert
        assert_eq!(actual, Some(9));
    }

    #[test]
    fn test_40001_single_pixel_noise_color_from_sparse_object() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 0, 5, 5, 5, 5,
            7, 7, 7, 0, 3, 5, 5, 5, 5,
            7, 7, 3, 0, 0, 5, 5, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            9, 8, 8, 8, 8, 8, 8, 5, 5,
        ];
        let input: Image = Image::try_create(9, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Option<u8> = objects.single_pixel_noise_color();

        // Assert
        assert_eq!(actual, Some(3));
    }

    #[test]
    fn test_40002_single_pixel_noise_color_from_sparse_object_ambiguous() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 0, 5, 5, 5, 5,
            7, 7, 7, 0, 3, 5, 5, 5, 5,
            7, 7, 3, 0, 0, 5, 5, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 5, 5,
            8, 8, 8, 8, 8, 8, 8, 9, 5,
            9, 8, 8, 8, 8, 8, 8, 5, 5,
        ];
        let input: Image = Image::try_create(9, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Option<u8> = objects.single_pixel_noise_color();

        // Assert
        assert_eq!(actual, None);
    }

    #[test]
    fn test_40003_single_pixel_noise_color_from_sparse_object_ignore_diagonal() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 0, 5, 3, 5, 3,
            7, 7, 7, 0, 0, 5, 5, 5, 5,
            7, 7, 7, 0, 0, 5, 5, 3, 5,
            8, 8, 8, 9, 8, 8, 8, 5, 5,
            8, 8, 9, 8, 8, 8, 8, 5, 5,
            8, 9, 8, 8, 8, 8, 8, 5, 5,
            9, 8, 8, 8, 8, 8, 8, 5, 5,
        ];
        let input: Image = Image::try_create(9, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Option<u8> = objects.single_pixel_noise_color();

        // Assert
        assert_eq!(actual, Some(3));
    }

    #[test]
    fn test_50000_mass_as_image_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 0, 5,
            7, 7, 7, 0, 0, 5,
            7, 7, 7, 0, 0, 5,
            8, 8, 8, 8, 5, 8,
            8, 8, 8, 5, 8, 8,
            8, 7, 8, 8, 8, 8,
            7, 8, 7, 8, 8, 8,
        ];
        let input: Image = Image::try_create(6, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.mass_as_image(PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
             9,  9,  9,  6,  6,  3,
             9,  9,  9,  6,  6,  3,
             9,  9,  9,  6,  6,  3,
            18, 18, 18, 18,  1, 18,
            18, 18, 18,  1, 18, 18,
            18,  1, 18, 18, 18, 18,
             1,  1,  1, 18, 18, 18,
        ];
        let expected: Image = Image::try_create(6, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_50001_mass_as_image_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            8, 8, 5, 8,
            8, 5, 8, 8,
            8, 8, 8, 8,
        ];
        let input: Image = Image::try_create(4, 3, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.mass_as_image(PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            10, 10,  1, 10,
            10,  1, 10, 10,
            10, 10, 10, 10,
        ];
        let expected: Image = Image::try_create(4, 3, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_60000_holes_mask_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 0, 0,
            5, 7, 7, 5, 3, 3,
            5, 5, 5, 5, 0, 0,
            5, 5, 2, 5, 0, 8,
            5, 5, 5, 5, 0, 0,
            5, 9, 9, 5, 0, 8,
            5, 5, 5, 5, 8, 8,
        ];
        let input: Image = Image::try_create(6, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.holes_mask(5, PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70000_holecount_image_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            5, 5, 5, 5, 0, 0,
            5, 7, 7, 5, 3, 3,
            5, 5, 5, 5, 0, 0,
            5, 5, 2, 5, 0, 8,
            5, 5, 5, 5, 0, 0,
            5, 9, 9, 5, 0, 8,
            5, 5, 5, 5, 8, 8,
        ];
        let input: Image = Image::try_create(6, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.holecount_image(5, PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            3, 3, 3, 3, 0, 0,
            3, 0, 0, 3, 0, 0,
            3, 3, 3, 3, 0, 0,
            3, 3, 0, 3, 0, 0,
            3, 3, 3, 3, 0, 0,
            3, 0, 0, 3, 0, 0,
            3, 3, 3, 3, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_70001_holecount_image_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 7, 0,
            0, 0, 3, 3, 3, 0,
            0, 0, 3, 0, 3, 0,
            0, 3, 3, 3, 3, 0,
            0, 3, 3, 0, 7, 0,
            7, 7, 7, 0, 7, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.holecount_image(3, PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0,
            0, 0, 1, 0, 1, 0,
            0, 1, 1, 1, 1, 0,
            0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_80000_filled_holes_mask_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 7, 0,
            0, 0, 3, 3, 3, 0,
            0, 0, 3, 0, 3, 0,
            0, 3, 3, 3, 3, 0,
            0, 3, 3, 0, 7, 0,
            7, 7, 7, 0, 7, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.filled_holes_mask(3, PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0,
            0, 0, 1, 1, 1, 0,
            0, 1, 1, 1, 1, 0,
            0, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_80001_filled_holes_mask_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            7, 7, 7, 0, 7, 0,
            0, 0, 3, 3, 3, 0,
            0, 0, 3, 0, 3, 0,
            0, 3, 3, 3, 3, 0,
            0, 3, 3, 0, 7, 0,
            7, 7, 7, 0, 7, 0,
        ];
        let input: Image = Image::try_create(6, 6, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.filled_holes_mask(7, PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            1, 1, 1, 0, 1, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0,
            1, 1, 1, 0, 1, 0,
        ];
        let expected: Image = Image::try_create(6, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_80002_filled_holes_mask_connectivity4() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 0, 3, 3, 3, 0,
            0, 0, 0, 0, 3, 0, 3, 0,
            3, 3, 3, 0, 3, 3, 3, 3,
            3, 0, 3, 0, 0, 3, 0, 3,
            3, 3, 3, 0, 3, 3, 3, 3,
            0, 3, 3, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let input: Image = Image::try_create(8, 7, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act
        let actual: Image = objects.filled_holes_mask(3, PixelConnectivity::Connectivity4).expect("image");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 1, 1, 1, 0,
            0, 0, 0, 0, 1, 1, 1, 0,
            1, 1, 1, 0, 1, 1, 1, 1,
            1, 1, 1, 0, 0, 1, 1, 1,
            1, 1, 1, 0, 1, 1, 1, 1,
            0, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(8, 7, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_90000_corner_classification() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 7, 0, 7,
            0, 6, 0, 0, 7, 0,
            0, 0, 0, 7, 0, 7,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act + Assert
        {
            assert_eq!(objects.corner_classification(0, 0, 0), 1);
            assert_eq!(objects.corner_classification(0, 5, 0), 2);
            assert_eq!(objects.corner_classification(0, 0, 2), 4);
            assert_eq!(objects.corner_classification(0, 5, 2), 8);
            assert_eq!(objects.corner_classification(6, 1, 1), 15);
            assert_eq!(objects.corner_classification(7, 3, 0), 1);
            assert_eq!(objects.corner_classification(7, 5, 0), 2);
            assert_eq!(objects.corner_classification(7, 3, 2), 4);
            assert_eq!(objects.corner_classification(7, 5, 2), 8);
        }
    }

    #[test]
    fn test_100000_is_inside_bounding_box() {
        // Arrange
        let pixels: Vec<u8> = vec![
            0, 0, 0, 7, 0, 7,
            0, 6, 0, 0, 7, 0,
            0, 0, 0, 7, 0, 7,
        ];
        let input: Image = Image::try_create(6, 3, pixels).expect("image");
        let objects: SingleColorObjects = SingleColorObjects::find_objects(&input).expect("ColorIsObject");
        
        // Act + Assert
        {
            // color 0
            assert_eq!(objects.is_inside_bounding_box(0, 0, 0), true);
            assert_eq!(objects.is_inside_bounding_box(0, 5, 0), true);
            assert_eq!(objects.is_inside_bounding_box(0, 0, 2), true);
            assert_eq!(objects.is_inside_bounding_box(0, 5, 2), true);
        }

        {
            // color 6
            assert_eq!(objects.is_inside_bounding_box(6, 0, 1), false);
            assert_eq!(objects.is_inside_bounding_box(6, 1, 1), true);
            assert_eq!(objects.is_inside_bounding_box(6, 2, 1), false);
        }

        {
            // color 7
            assert_eq!(objects.is_inside_bounding_box(7, 2, 0), false);
            assert_eq!(objects.is_inside_bounding_box(7, 3, 0), true);
            assert_eq!(objects.is_inside_bounding_box(7, 5, 0), true);
            assert_eq!(objects.is_inside_bounding_box(7, 3, 2), true);
            assert_eq!(objects.is_inside_bounding_box(7, 5, 2), true);
        }
    }
}
