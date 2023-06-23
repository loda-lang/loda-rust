//! Perform gravity operations on objects matching shapes into the corresponding holes.
//! 
//! The ARC task `6a1e5592` is an example of this.
use super::{Image, ImageMask, ImageMaskCount, ImageSize, ImageOverlay, ImageReplaceColor, MixMode, ImageMix, ImageSymmetry, ImageRotate, ImageMaskBoolean, Rectangle, ImageCrop, PixelConnectivity, ImageMaskGrow, ImageCompare, ImageMaskSolidGround};

#[allow(unused_imports)]
use super::{HtmlLog, ImageToHTML, ImageLabel, GridLabel};

static VERBOSE_GRAVITY: bool = false;

#[allow(dead_code)]
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
    #[allow(dead_code)]
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
                HtmlLog::image(&item.mask_cropped);
            }
            items.push(item);
        }
        if items.is_empty() {
            return Err(anyhow::anyhow!("ObjectsAndGravity.new: found zero objects. There must be 1 or more objects"));
        }
        if VERBOSE_GRAVITY {
            HtmlLog::text(format!("Found {} objects", items.len()));
        }
        let instance = Self {
            image_size: enumerated_objects.size(),
            items
        };
        Ok(instance)
    }

    /// Pick an object from the list of objects that have not been placed yet.
    /// 
    /// Returns a tuple:
    /// - An image with the object at its new position.
    /// - The `object_id` of the object that was placed.
    fn gravity_single_object(&self, solid_mask: &Image) -> anyhow::Result<(Image, u8)> {
        if solid_mask.size() != self.image_size {
            return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: solid_mask.size() != self.image_size"));
        }
        let solid_mask_count: u16 = solid_mask.mask_count_one();
        let solid_mask_grow: Image = solid_mask.mask_grow(PixelConnectivity::Connectivity8)?;
        let solid_outline_mask: Image = solid_mask_grow.diff(&solid_mask)?;

        // Empty pixels that you can stand on, that has a solid object immediately below it.
        let solid_ground_below_mask: Image = solid_mask.mask_ground_below()?;

        let mut candidate_vec = Vec::<Candidate>::new();

        // Identify all positions where each object can be placed
        // The object that has the fewest positions, is possible the object that needs to be placed first.
        for (item_index, item) in self.items.iter().enumerate() {
            if item.has_been_placed {
                continue;
            }
            let object_mass: u16 = item.object_mass;
            let bounding_box_mass: u16 = item.bounding_box.width() as u16 * item.bounding_box.height() as u16;
            let mut score_verbose: Image = if VERBOSE_GRAVITY { 
                Image::zero(self.image_size.width, self.image_size.height) 
            } else { 
                Image::empty()
            };
            let correct_count: u16 = solid_mask_count + item.object_mass;
            let score_factor: u16 = (item.mask_cropped.width() as u16) * (item.mask_cropped.height() as u16);
            let mut found_distance_to_bottom: u8 = u8::MAX;
            let mut highest_score: u16 = 0;
            let mut positions_unfiltered = Vec::<CandidatePosition>::new();
            for x in 0..self.image_size.width {

                // Traverse from the bottom to the top. And probe if the object can be placed at this position.
                // The moment a good spot is found, then register the y-position and move on to the next x-position.
                for y_reverse in 0..self.image_size.height {
                    let y: u8 = ((self.image_size.height as i32) - (y_reverse as i32) - 1).min(255).max(0) as u8;
                    let candidate_mask: Image = solid_mask.overlay_with_mask_and_position(&item.mask_cropped, &item.mask_cropped, x as i32, y as i32)?;
                    let candidate_mask_count: u16 = candidate_mask.mask_count_one();
                    if candidate_mask_count != correct_count {
                        // println!("object {} position: {} {}  mismatch in mass: {} != {}", index, x, y, candidate_mask_count, correct_count);
                        continue;
                    }
                    let intersection: Image = candidate_mask.mask_and(&solid_outline_mask)?;
                    let intersection_count0: u16 = intersection.mask_count_one();
                    let intersection_count1: u16 = intersection_count0 + 1;
                    let score_value: u16 = intersection_count1 * score_factor * (y_reverse as u16);

                    // Measure number of holes underneath the object
                    let intersection_touch: Image = candidate_mask.mask_and(&solid_ground_below_mask)?;
                    let ground_touch_count: u8 = intersection_touch.mask_count_one().min(255) as u8;
                    let ground_notouch_count: u8 = ((item.bounding_box.width() as i32) - (ground_touch_count as i32)).max(0) as u8;

                    if VERBOSE_GRAVITY {
                        score_verbose.set(x as i32, y as i32, intersection_count0.min(255) as u8);
                    }
                    highest_score = highest_score.max(score_value);
                    let distance_to_bottom: u8 = ((y_reverse as i32) + (item.bounding_box.height() as i32) - 1).min(255) as u8;
                    found_distance_to_bottom = found_distance_to_bottom.min(distance_to_bottom);
                    let mut candidate_position = CandidatePosition { 
                        x,
                        y,
                        distance_to_bottom, 
                        intersection_count0, 
                        ground_touch_count, 
                        ground_notouch_count, 
                        object_mass,
                        bounding_box_mass,
                        computed_score: 0 
                    };
                    candidate_position.assign_score();
                    positions_unfiltered.push(candidate_position);
                    break;
                }
            }
            let mut positions_filtered = Vec::<CandidatePosition>::new();
            for position in &positions_unfiltered {
                if position.distance_to_bottom == found_distance_to_bottom {
                    positions_filtered.push(position.clone());
                }
            }
            positions_filtered.sort_unstable_by_key(|position| (position.computed_score, position.x));

            let best_position: CandidatePosition = match positions_filtered.last() {
                Some(position) => position.clone(),
                None => {
                    return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: unable to find a position"));
                }
            };
            
            if VERBOSE_GRAVITY {
                let mut position_visualization: Image = Image::zero(self.image_size.width, self.image_size.height);
                for position in &positions_filtered {
                    if VERBOSE_GRAVITY {
                        if position.x == best_position.x {
                            HtmlLog::text(format!("position: {:?} -- BEST", position));
                        } else {
                            HtmlLog::text(format!("position: {:?}", position));
                        }
                    }
                    position_visualization.set(position.x as i32, position.y as i32, 1);
                }
                println!("item_index {} highest_score: {} found_distance_to_bottom: {} score: {:?}", item_index, highest_score, found_distance_to_bottom, score_verbose);
                HtmlLog::image(&score_verbose);
                HtmlLog::image(&position_visualization);
            }
            let candidate = Candidate { 
                item_index, 
                highest_y: found_distance_to_bottom, 
                best_position,
            };
            candidate_vec.push(candidate);
        }
        if VERBOSE_GRAVITY {
            HtmlLog::text(format!("candidate_vec.len() {}", candidate_vec.len()));
        }

        // Pick the candidate with highest score
        let candidate: Candidate;
        {
            let mut candidate_vec2 = candidate_vec.clone();
            candidate_vec2.sort_unstable_by_key(|candidate| (candidate.score(), candidate.item_index));
            
            candidate = match candidate_vec2.last() {
                Some(value) => value.clone(),
                None => return Err(anyhow::anyhow!("ObjectsAndGravity.gravity: no candidate found")),
            };
        }
        if VERBOSE_GRAVITY {
            println!("candidate.item_index: {}", candidate.item_index);
        }
        
        let found_y: i32 = candidate.best_position.y as i32;
        let found_x: i32 = candidate.best_position.x as i32;
        if VERBOSE_GRAVITY {
            println!("found_x: {} found_y: {}", found_x, found_y);
        }

        let mut result_image: Image = Image::zero(self.image_size.width, self.image_size.height);

        // place the object
        let item: &Item = &self.items[candidate.item_index];
        let object_to_draw: Image = item.mask_cropped.replace_color(1, item.object_id)?;
        result_image = result_image.overlay_with_mask_and_position(&object_to_draw, &item.mask_cropped, found_x, found_y)?;

        if VERBOSE_GRAVITY {
            HtmlLog::text(format!("did place object {} at ({},{})", item.index, found_x, found_y));
        }

        Ok((result_image, item.object_id))
    }

    /// loop through all the objects, and applies gravity to them.
    /// 
    /// Pick weirdest shape first, that is hard to fit into the holes, and then
    /// progress towards smaller shapes that are square and easier to place.
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

            let (image, object_id) = match self.gravity_single_object(&solid_mask_accumulated) {
                Ok(value) => value,
                Err(error) => {
                    println!("gravity_multiple_objects: Unable to place single object. error: {}", error);
                    break;
                },
            };

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
    item_index: usize,
    highest_y: u8,
    best_position: CandidatePosition,
}

impl Candidate {
    fn score(&self) -> i32 {
        let a: i32 = (self.highest_y as i32) * 10000 + self.best_position.computed_score;
        a
    }
}

#[derive(Clone, Debug)]
struct CandidatePosition {
    /// Pick an position that is as close to the original position as possible.
    x: u8,

    /// Place the object at the deepest possible position.
    y: u8,

    /// Minimize. The closer to the bottom the better.
    distance_to_bottom: u8,
    
    /// Maximize. As many pixels should be touching the ground as possible.
    ground_touch_count: u8,
    
    /// Minimize. If the object shape leaves holes underneath then it's bad.
    ground_notouch_count: u8,

    /// Maximize. How many pixels intersect with the outline of the solid ground mask.
    intersection_count0: u16,

    /// Start out with the biggest and most complex objects, and progress towards easier objects.
    object_mass: u16,
    bounding_box_mass: u16,

    computed_score: i32,

    // Future experiments
    // Complexity of the object bottom. The more complex the more important it is to find a good fit.
}

impl CandidatePosition {
    fn assign_score(&mut self) {
        self.computed_score = self.score();
    }

    fn score(&self) -> i32 {
        if self.ground_touch_count < 1 {
            // The object doesn't touch the ground. It's a terrible candidate.
            return 0;
        }
        
        // Determine how much of the object is touching the ground. The more the better.
        // Holes underneath the object are unwanted.
        let numerator: f32 = self.ground_touch_count as f32;
        let denominator: f32 = self.ground_touch_count as f32 + self.ground_notouch_count as f32;
        let jaccard_index: f32 = numerator / denominator;

        // Scale up the score by how connected it is to the left/right structure.
        // Scale up the score by how big the object is.
        let score: f32 = jaccard_index * (self.intersection_count0 as f32) * (self.bounding_box_mass as f32) / (self.object_mass as f32);

        // Convert to integer, so the candidates can be sorted by their score.
        (score * 10000.0) as i32
    }
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
