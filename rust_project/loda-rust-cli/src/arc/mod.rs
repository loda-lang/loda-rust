//! ARC challenge experiments
mod arc_json_model;
mod bitmap;
mod bitmap_to_number;
mod bitmap_try_create;
mod convolution3x3;
mod convolution_with_program;
mod index_for_pixel;
mod padding;
mod number_to_bitmap;
mod read_testdata;
mod test_convert;

pub use bitmap::Bitmap;
pub use bitmap_to_number::BitmapToNumber;
pub use bitmap_try_create::BitmapTryCreate;
pub use convolution3x3::convolution3x3;
pub use number_to_bitmap::NumberToBitmap;
pub use padding::Padding;
pub use read_testdata::read_testdata;
