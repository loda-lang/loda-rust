//! Perform gravity operations on objects
use super::{Image, ImageMask, ImageMaskCount, ImageSize, ImageOverlay, ImageTrim, ImageReplaceColor, MixMode, ImageMix, ImageSymmetry, ImageRotate, ImageOutline, ImageMaskBoolean, Rectangle, ImageCrop};

static VERBOSE_GRAVITY: bool = true;

pub enum ObjectsAndGravityDirection {
    GravityUp,
    GravityDown,
    GravityLeft,
    GravityRight,
}

#[allow(dead_code)]
pub struct ObjectsAndGravity {
    image_size: ImageSize,
    items: Vec<Item>,
}

impl ObjectsAndGravity {
    pub fn gravity(enumerated_objects: &Image, solid_mask: &Image, direction: ObjectsAndGravityDirection) -> anyhow::Result<Image> {
        if enumerated_objects.size() != solid_mask.size() {
            return Err(anyhow::anyhow!("both images must be the same size"));
        }

        let transformed_enumerated_objects: Image = match direction {
            ObjectsAndGravityDirection::GravityUp => enumerated_objects.flip_y()?,
            ObjectsAndGravityDirection::GravityDown => enumerated_objects.clone(),
            ObjectsAndGravityDirection::GravityLeft => enumerated_objects.rotate_cw()?,
            ObjectsAndGravityDirection::GravityRight => enumerated_objects.rotate_ccw()?,
        };

        let transformed_solid_mask: Image = match direction {
            ObjectsAndGravityDirection::GravityUp => solid_mask.flip_y()?,
            ObjectsAndGravityDirection::GravityDown => solid_mask.clone(),
            ObjectsAndGravityDirection::GravityLeft => solid_mask.rotate_cw()?,
            ObjectsAndGravityDirection::GravityRight => solid_mask.rotate_ccw()?,
        };

        let mut instance = Self::new(&transformed_enumerated_objects)?;
        let result_image = instance.gravity_multiple_objects(&transformed_solid_mask)?;

        let transformed_result_image: Image = match direction {
            ObjectsAndGravityDirection::GravityUp => result_image.flip_y()?,
            ObjectsAndGravityDirection::GravityDown => result_image.clone(),
            ObjectsAndGravityDirection::GravityLeft => result_image.rotate_ccw()?,
            ObjectsAndGravityDirection::GravityRight => result_image.rotate_cw()?,
        };
        Ok(transformed_result_image)
    }

    /// Extracts the objects, and returns an instance of `ObjectsAndGravity`.
    /// 
    /// The `enumerated_objects` must be 1x1 or bigger.
    /// 
    /// An error is returned if there are zero objects.
    #[allow(dead_code)]
    fn new(enumerated_objects: &Image) -> anyhow::Result<Self> {
        if enumerated_objects.is_empty() {
            return Err(anyhow::anyhow!("ObjectsAndGravity.new: image must be 1x1 or bigger"));
        }
        let mut items = Vec::<Item>::new();
        // Skip over color 0. It's reserved for the background, and is not considered an object.
        for color in 1..=255u8 {
            let mask_uncropped: Image = enumerated_objects.to_mask_where_color_is(color);
            let mass_of_object: u16 = mask_uncropped.mask_count_one();
            if mass_of_object == 0 {
                continue;
            }
            let bounding_box: Rectangle = match mask_uncropped.bounding_box() {
                Some(value) => value,
                None => {
                    if VERBOSE_GRAVITY {
                        println!("Integrity error. cannot find bounding box of a mask that contains non-zero pixels. should not happen.");
                    }
                    continue;
                }
            };
            let mask_cropped: Image = mask_uncropped.crop(bounding_box)?;
            let item = Item {
                index: items.len(),
                object_id: color,
                mask_cropped,
                object_mass: mass_of_object,
                has_been_placed: false,
                bounding_box,
            };
            if VERBOSE_GRAVITY {
                println!("index: {} color {} mass: {} mask: {:?}", item.index, item.object_id, item.object_mass, item.mask_cropped);
            }
            items.push(item);
        }
        if items.is_empty() {
            return Err(anyhow::anyhow!("ObjectsAndGravity.new: found zero objects. There must be 1 or more objects"));
        }
        let instance = Self {
            image_size: enumerated_objects.size(),
            items
        };
        Ok(instance)
    }

    #[allow(dead_code)]
    fn gravity_single_object(&self, solid_mask: &Image) -> anyhow::Result<(Image, u8)> {
        if solid_mask.size() != self.image_size {
            return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: solid_mask.size() != self.image_size"));
        }
        let solid_mask_count: u16 = solid_mask.mask_count_one();
        let solid_outline_mask: Image = solid_mask.outline_mask_neighbour()?;

        let mut candidate_vec = Vec::<Candidate>::new();

        // Identify all positions where each object can be placed
        // The object that has the fewest positions, is possible the object that needs to be placed first.
        for (item_index, item) in self.items.iter().enumerate() {
            if item.has_been_placed {
                continue;
            }
            let mut score: Image = Image::zero(self.image_size.width, self.image_size.height);
            let correct_count: u16 = solid_mask_count + item.object_mass;
            let score_factor: u16 = (item.mask_cropped.width() as u16) * (item.mask_cropped.height() as u16);
            let mut highest_score: u16 = 0;
            for x in 0..self.image_size.width {
                for y in 0..self.image_size.height {
                    let y_reverse: i32 = (self.image_size.height as i32) - (y as i32) - 1;
                    let candidate_mask: Image = solid_mask.overlay_with_mask_and_position(&item.mask_cropped, &item.mask_cropped, x as i32, y_reverse)?;
                    let candidate_mask_count: u16 = candidate_mask.mask_count_one();
                    if candidate_mask_count != correct_count {
                        // println!("object {} position: {} {}  mismatch in mass: {} != {}", index, x, y, candidate_mask_count, correct_count);
                        continue;
                    }
                    let intersection: Image = candidate_mask.mask_and(&solid_outline_mask)?;
                    let intersection_count: u16 = intersection.mask_count_one() + 1;
                    let score_value: u16 = intersection_count * score_factor * (y as u16);
                    let score_value_clamped: u8 = score_value.min(u8::MAX as u16) as u8;
                    // println!("object {} position: {} {}", index, x, y);
                    score.set(x as i32, y_reverse, score_value_clamped);
                    highest_score = highest_score.max(score_value);
                }
            }
            if VERBOSE_GRAVITY {
                println!("item_index {} highest_score: {} score: {:?}", item_index, highest_score, score);
            }
            let mass1: u16 = score.mask_count_nonzero();
            let mass2: u16 = (item.mask_cropped.width() as u16) * (item.mask_cropped.height() as u16);
            // let mass: u16 = mass1 * mass2;
            let mass: u16 = mass1;
            candidate_vec.push(Candidate { score, mass, item_index, highest_score });
        }

        // Pick the candidate with the lowest mass, which is the fewest number of positions where the object can fit
        let mut found_candidate: Option<&Candidate> = None;
        let mut count_ambiguous: u16 = 0;
        if false {
            let mut lowest_mass: u16 = u16::MAX;
            for candidate in &candidate_vec {
                if candidate.mass > lowest_mass {
                    continue;
                }
                if lowest_mass == candidate.mass {
                    count_ambiguous += 1;
                } else {
                    count_ambiguous = 0;
                }
                lowest_mass = candidate.mass;
                found_candidate = Some(candidate);
            }
            if count_ambiguous > 0 {
                return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: ambiguous what object to pick. lowest_mass {}", lowest_mass));
            }
        }
        if true {
            let mut highest_score: u16 = 0;
            for candidate in &candidate_vec {
                if candidate.highest_score < highest_score {
                    continue;
                }
                if candidate.highest_score == highest_score {
                    count_ambiguous += 1;
                } else {
                    count_ambiguous = 0;
                }
                highest_score = candidate.highest_score;
                found_candidate = Some(candidate);
            }
            if count_ambiguous > 0 {
                return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: ambiguous what object to pick highest_score: {}", highest_score));
            }
        }
        if count_ambiguous > 0 {
            return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: ambiguous what object to pick"));
        }
        let candidate: Candidate = match found_candidate {
            Some(value) => value.clone(),
            None => return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: no candidate found")),
        };
        if VERBOSE_GRAVITY {
            println!("candidate.item_index: {}", candidate.item_index);
        }

        if candidate.score.size() != self.image_size {
            return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: integrity error. the candidate.score.size() != self.image_size"));
        }
        
        let score: &Image = &candidate.score;
        let mut found_y: i32 = -1;
        let mut found_x: i32 = -1;
        let mut count_ambiguous2: u16 = 0;
        for x in 0..score.width() as i32 {
            for y in 0..score.height() as i32 {
                if y < found_y {
                    continue;
                }
                let value: u8 = score.get(x, y).unwrap_or(0);
                if value == 0 {
                    continue;
                }
                if y == found_y {
                    count_ambiguous2 += 1;
                } else {
                    count_ambiguous2 = 0;
                }
                found_y = y;
                found_x = x;
            }
        }
        if VERBOSE_GRAVITY {
            println!("found_x: {} found_y: {} count_ambiguous2: {}", found_x, found_y, count_ambiguous2);
        }
        if count_ambiguous2 > 0 {
            return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: ambiguous what position to pick"));
        }
        if found_x < 0 || found_y < 0 {
            return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: did not find a position to place the object"));
        }

        let mut result_image: Image = Image::zero(self.image_size.width, self.image_size.height);

        // place the object
        let item: &Item = &self.items[candidate.item_index];
        let object_to_draw: Image = item.mask_cropped.replace_color(1, item.object_id)?;
        result_image = result_image.overlay_with_mask_and_position(&object_to_draw, &item.mask_cropped, found_x, found_y)?;

        Ok((result_image, item.object_id))
    }

    /// loop that goes through all the objects, and applies gravity to them.
    /// 
    /// Pick weirdest shape first, that are hard to fit into the holes, and then
    /// progress towards smaller shapes that are square and easier to place.
    #[allow(dead_code)]
    fn gravity_multiple_objects(&mut self, solid_mask: &Image) -> anyhow::Result<Image> {
        let mut result_image: Image = Image::zero(self.image_size.width, self.image_size.height);
        let mut solid_mask_accumulated: Image = solid_mask.clone();
        for i in 0..self.items.len() {
            let mut has_all_been_placed: bool = true;
            for item in &self.items {
                if !item.has_been_placed {
                    has_all_been_placed = false;
                    break;
                }
            }
            if has_all_been_placed {
                break;
            }

            let (image, object_id) = self.gravity_single_object(&solid_mask_accumulated)?;

            for item in self.items.iter_mut() {
                if item.object_id == object_id {
                    item.has_been_placed = true;
                    if VERBOSE_GRAVITY {
                        println!("iteration: {} did place object {}", i, object_id);
                    }
                }
            }

            let mask: Image = image.to_mask_where_color_is_different(0);
            solid_mask_accumulated = solid_mask_accumulated.mix(&mask, MixMode::Plus)?;

            // Detect if there are overlapping objects
            let overlap_mask: Image = solid_mask_accumulated.to_mask_where_color_is_equal_or_greater_than(2);
            if overlap_mask.mask_count_one() > 0 {
                return Err(anyhow::anyhow!("ObjectsAndGravity.gravity_multiple_objects: integrity error. placed the object on top of another object"));
            }

            result_image = result_image.overlay_with_mask_color(&image, 0)?;
        }
        Ok(result_image)
    }
}

#[derive(Clone, Debug)]
struct Candidate {
    score: Image, 
    mass: u16,
    item_index: usize,
    highest_score: u16,
}

#[derive(Clone, Debug)]
struct Item {
    index: usize,
    object_id: u8,
    bounding_box: Rectangle,
    mask_cropped: Image,
    object_mass: u16,
    has_been_placed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_gravity_single_object() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 1, 0, 0, 7, 0, 0,
            1, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 7, 7, 0, 2,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(7, 6, enumerated_object_pixels).expect("image");
        let instance: ObjectsAndGravity = ObjectsAndGravity::new(&enumerated_objects).expect("ok");

        let solid_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 1, 0, 0,
            0, 1, 0, 0, 1, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let solid: Image = Image::try_create(7, 6, solid_pixels).expect("image");

        // Act
        let (actual, object_id) = instance.gravity_single_object(&solid).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0,
            0, 0, 7, 7, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(7, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
        assert_eq!(object_id, 7);
    }

    #[test]
    fn test_20000_gravity_multiple_objects() {
        // Arrange
        let enumerated_object_pixels: Vec<u8> = vec![
            1, 1, 0, 0, 7, 0, 0,
            1, 0, 0, 0, 7, 0, 0,
            0, 0, 0, 7, 7, 0, 2,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let enumerated_objects: Image = Image::try_create(7, 6, enumerated_object_pixels).expect("image");

        let solid_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 1, 0, 0,
            0, 1, 0, 0, 1, 0, 1,
            1, 1, 1, 1, 1, 1, 1,
        ];
        let solid: Image = Image::try_create(7, 6, solid_pixels).expect("image");

        // Act
        let mut instance: ObjectsAndGravity = ObjectsAndGravity::new(&enumerated_objects).expect("ok");
        let actual: Image = instance.gravity_multiple_objects(&solid).expect("ok");

        // Assert
        let expected_pixels: Vec<u8> = vec![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 7, 0, 0, 0,
            0, 0, 0, 7, 0, 1, 1,
            2, 0, 7, 7, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let expected: Image = Image::try_create(7, 6, expected_pixels).expect("image");
        assert_eq!(actual, expected);
    }
}
