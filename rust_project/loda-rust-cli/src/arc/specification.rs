use super::{Image, ImageHistogram};

/// Future experiment:
/// In an ARC task. Give each test pair a `Specification` with what the output is supposed to be.
/// If the `Specification` is not satisfied, the prediction can be rejected.
#[allow(dead_code)]
trait Specification<T> {
    /// Returns either `true` or `false` if the specification is satisfied.
    ///
    /// The specification may return an error when encountering something unexpected.
    fn is_satisfied_by(&self, candidate: &T) -> anyhow::Result<bool>;

    fn and<O: Specification<T>>(self, other: O) -> AndCombination<Self, O>
    where Self: Sized {
        AndCombination(self, other)
    }

    fn or<O: Specification<T>>(self, other: O) -> OrCombination<Self, O>
    where Self: Sized {
        OrCombination(self, other)
    }
}

struct AndCombination<L, R>(L,R);

impl<T, L, R> Specification<T> for AndCombination<L,R>
    where L: Specification<T>,
        R: Specification<T>
{
    fn is_satisfied_by(&self, candidate: &T) -> anyhow::Result<bool> {
        let left: bool = self.0.is_satisfied_by(candidate)?; 
        let right: bool = self.1.is_satisfied_by(candidate)?;
        Ok(left && right)
    }
}

struct OrCombination<L, R>(L,R);

impl<T, L, R> Specification<T> for OrCombination<L,R>
    where L: Specification<T>,
        R: Specification<T>
{
    fn is_satisfied_by(&self, candidate: &T) -> anyhow::Result<bool> {
        let left: bool = self.0.is_satisfied_by(candidate)?; 
        let right: bool = self.1.is_satisfied_by(candidate)?;
        Ok(left || right)
    }
}

struct ReturnValueSpecification {
    value: bool,
}

impl<T> Specification<T> for ReturnValueSpecification {
    fn is_satisfied_by(&self, _candidate: &T) -> anyhow::Result<bool> {
        Ok(self.value)
    }
}

struct TrueSpecification;

impl<T> Specification<T> for TrueSpecification {
    fn is_satisfied_by(&self, _candidate: &T) -> anyhow::Result<bool> {
        Ok(true)
    }
}

struct FalseSpecification;

impl<T> Specification<T> for FalseSpecification {
    fn is_satisfied_by(&self, _candidate: &T) -> anyhow::Result<bool> {
        Ok(false)
    }
}

#[allow(dead_code)]
struct TwoOrMoreUniqueColorsSpecification;

impl Specification<Image> for TwoOrMoreUniqueColorsSpecification {
    fn is_satisfied_by(&self, candidate: &Image) -> anyhow::Result<bool> {
        let count: u16 = candidate.histogram_all().number_of_counters_greater_than_zero();
        Ok(count >= 2)
    }
}

#[allow(dead_code)]
struct ImageSizeSpecification {
    width: u8,
    height: u8,
}

impl ImageSizeSpecification {
    #[allow(dead_code)]
    fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }
}

impl Specification<Image> for ImageSizeSpecification {
    fn is_satisfied_by(&self, candidate: &Image) -> anyhow::Result<bool> {
        let same_width: bool = candidate.width() == self.width;
        let same_height: bool = candidate.height() == self.height;
        Ok(same_width && same_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arc::ImageTryCreate;

    #[test]
    fn test_10000_and_specification() {
        {
            let a = FalseSpecification;
            let b = FalseSpecification;
            let c = Specification::<u8>::and(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, false);
        }
        {
            let a = FalseSpecification;
            let b = TrueSpecification;
            let c = Specification::<u8>::and(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, false);
        }
        {
            let a = TrueSpecification;
            let b = FalseSpecification;
            let c = Specification::<u8>::and(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, false);
        }
        {
            let a = TrueSpecification;
            let b = TrueSpecification;
            let c = Specification::<u8>::and(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, true);
        }
    }

    #[test]
    fn test_20000_or_specification() {
        {
            let a = FalseSpecification;
            let b = FalseSpecification;
            let c = Specification::<u8>::or(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, false);
        }
        {
            let a = FalseSpecification;
            let b = TrueSpecification;
            let c = Specification::<u8>::or(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, true);
        }
        {
            let a = TrueSpecification;
            let b = FalseSpecification;
            let c = Specification::<u8>::or(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, true);
        }
        {
            let a = TrueSpecification;
            let b = TrueSpecification;
            let c = Specification::<u8>::or(a, b);
            let actual: bool = c.is_satisfied_by(&0).expect("ok");
            assert_eq!(actual, true);
        }
    }

    #[test]
    fn test_30000_two_or_more_unique_colors() {
        {
            // Arrange
            let input: Image = Image::color(3, 3, 7);
    
            // Act
            let actual: bool = TwoOrMoreUniqueColorsSpecification.is_satisfied_by(&input).expect("ok");
    
            // Assert
            assert_eq!(actual, false);
        }
        {
            // Arrange
            let input: Image = Image::try_create(1, 2, vec![5, 7]).expect("image");

            // Act
            let actual: bool = TwoOrMoreUniqueColorsSpecification.is_satisfied_by(&input).expect("ok");

            // Assert
            assert_eq!(actual, true);
        }
    }

    #[test]
    fn test_40000_image_size() {
        {
            // Arrange
            let input: Image = Image::color(4, 3, 7);
    
            // Act
            let actual: bool = ImageSizeSpecification::new(3, 4).is_satisfied_by(&input).expect("ok");
    
            // Assert
            assert_eq!(actual, false);
        }
        {
            // Arrange
            let input: Image = Image::color(3, 4, 7);

            // Act
            let actual: bool = ImageSizeSpecification::new(3, 4).is_satisfied_by(&input).expect("ok");

            // Assert
            assert_eq!(actual, true);
        }
    }
}
