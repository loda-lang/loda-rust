//! Experiments with Abstraction and Reasoning Corpus (ARC)
mod action_label_util;
mod arc_json_model;
mod arc_json_model_to_html;
mod arc_puzzles;
mod arc_work_model;
mod arc_work_model_from_json_model;
mod arc_work_model_impl_imagemeta;
mod arc_work_model_impl_input;
mod arc_work_model_impl_object;
mod arc_work_model_impl_output;
mod arc_work_model_impl_pair;
mod arc_work_model_impl_task;
mod arcathon_solution_json;
mod auto_repair_symmetry;
mod center_of_mass;
mod color;
mod connected_component;
mod convolution2x2;
mod convolution3x3;
mod convolution3x3_with_mask;
mod convolution5x5_special;
mod convolution_with_program;
mod experiment_with_convolution;
mod export_tasks;
mod generate_pattern;
mod grid;
mod grid_to_label;
mod histogram;
mod html_log;
mod image;
mod image_border;
mod image_color_profile;
mod image_collect;
mod image_compare;
mod image_corner_analyze;
mod image_count_unique_colors;
mod image_crop;
mod image_denoise;
mod image_detect_color_symmetry;
mod image_detect_hole;
mod image_draw_line_where;
mod image_drawrect;
mod image_extract_rowcolumn;
mod image_fill;
mod image_find;
mod image_gravity;
mod image_grid;
mod image_histogram;
mod image_layout;
mod image_mask;
mod image_mask_boolean;
mod image_mask_count;
mod image_mask_distance;
mod image_mask_grow;
mod image_mask_solid_ground;
mod image_mix;
mod image_neighbour;
mod image_noise_color;
mod image_object_enumerate;
mod image_offset;
mod image_outline;
mod image_overlay;
mod image_padding;
mod image_palette;
mod image_periodicity;
mod image_position;
mod image_remove_duplicates;
mod image_remove_rowcolumn;
mod image_repair_offset;
mod image_repair_pattern;
mod image_repair_symmetry;
mod image_repair_trigram;
mod image_repeat;
mod image_replace_color;
mod image_replace_pattern;
mod image_replace_regex;
mod image_replace_simple;
mod image_resize;
mod image_rotate;
mod image_set_pixel_where;
mod image_size;
mod image_sort;
mod image_split;
mod image_stack;
mod image_stats;
mod image_symmetry;
mod image_tile;
mod image_to_html;
mod image_to_number;
mod image_trim;
mod image_try_create;
mod image_unicode_formatting;
mod inspect_predicted;
mod index_for_pixel;
mod inspect_task;
mod lab_analyze_task;
mod label;
mod ngram;
mod number_to_image;
mod object_with_different_color;
mod object_with_smallest_value;
mod objects_and_gravity;
mod objects_and_mass;
mod objects_and_position;
mod objects_measure_mass;
mod objects_sort_by_property;
mod objects_to_grid;
mod objects_unique_color_count;
mod output_specification;
mod prompt;
mod prompt_position;
mod prompt_run_length_encoding;
mod prompt_shape_transform;
mod pixel_connectivity;
mod popular_objects;
mod read_testdata;
mod rectangle;
mod register_arc_functions;
mod reverse_color_popularity;
mod run_with_program;
mod shape_identification;
mod shape_identification_from_single_color_object;
mod single_color_object;
mod single_color_object_satisfies_label;
mod single_color_object_to_label;
mod split;
mod stack_strings;
mod subcommand_arc_size;
mod subcommand_arc_web;
mod substitution_rule;
mod symmetry;
mod symmetry_to_label;
mod task_graph;
mod test_convert;
mod time_util;
mod traverse_programs_and_models;

pub use action_label_util::ActionLabelUtil;
pub use arc_json_model_to_html::ModelToHTML;
pub use arcathon_solution_json::{Prediction, TestItem, TaskItem, Tasks};
pub use auto_repair_symmetry::AutoRepairSymmetry;
pub use center_of_mass::CenterOfMass;
pub use color::Color;
pub use connected_component::{ConnectedComponent, ConnectedComponentItem};
pub use convolution2x2::convolution2x2;
pub use convolution3x3::convolution3x3;
pub use convolution3x3_with_mask::convolution3x3_with_mask;
pub use convolution5x5_special::convolution5x5_special;
pub use experiment_with_convolution::ExperimentWithConvolution;
pub use export_tasks::ExportTasks;
pub use generate_pattern::GeneratePattern;
pub use grid::{Grid, GridPattern};
pub use grid_to_label::GridToLabel;
pub use histogram::{Histogram, HistogramPair};
pub use html_log::HtmlLog;
pub use image::Image;
pub use image_border::ImageBorder;
pub use image_color_profile::ImageColorProfile;
pub use image_collect::ImageCollect;
pub use image_compare::ImageCompare;
pub use image_corner_analyze::ImageCornerAnalyze;
pub use image_count_unique_colors::ImageCountUniqueColors;
pub use image_crop::ImageCrop;
pub use image_denoise::ImageDenoise;
pub use image_detect_color_symmetry::{ImageDetectColorSymmetry, ImageDetectColorSymmetryMode};
pub use image_detect_hole::ImageDetectHole;
pub use image_drawrect::ImageDrawRect;
pub use image_draw_line_where::ImageDrawLineWhere;
pub use image_extract_rowcolumn::ImageExtractRowColumn;
pub use image_fill::ImageFill;
pub use image_find::ImageFind;
pub use image_gravity::{GravityDirection, ImageGravity};
pub use image_grid::ImageGrid;
pub use image_histogram::ImageHistogram;
pub use image_layout::{ImageLayout, ImageLayoutMode};
pub use image_mask::ImageMask;
pub use image_mask_boolean::ImageMaskBoolean;
pub use image_mask_count::ImageMaskCount;
pub use image_mask_distance::ImageMaskDistance;
pub use image_mask_grow::ImageMaskGrow;
pub use image_mask_solid_ground::ImageMaskSolidGround;
pub use image_mix::{ImageMix, MixMode};
pub use image_neighbour::{ImageNeighbour, ImageNeighbourDirection};
pub use image_noise_color::ImageNoiseColor;
pub use image_object_enumerate::ImageObjectEnumerate;
pub use image_offset::ImageOffset;
pub use image_outline::ImageOutline;
pub use image_overlay::ImageOverlay;
pub use image_padding::ImagePadding;
pub use image_palette::ImageCreatePalette;
pub use image_periodicity::ImagePeriodicity;
pub use image_position::ImagePosition;
pub use image_remove_duplicates::ImageRemoveDuplicates;
pub use image_remove_rowcolumn::ImageRemoveRowColumn;
pub use image_repair_offset::ImageRepairOffset;
pub use image_repair_pattern::ImageRepairPattern;
pub use image_repair_symmetry::ImageRepairSymmetry;
pub use image_repair_trigram::ImageRepairTrigram;
pub use image_repeat::ImageRepeat;
pub use image_replace_color::ImageReplaceColor;
pub use image_replace_pattern::ImageReplacePattern;
pub use image_replace_regex::{ImageReplaceRegex, ImageReplaceRegexToColor};
pub use image_replace_simple::ImageReplaceSimple;
pub use image_resize::ImageResize;
pub use image_rotate::ImageRotate;
pub use image_set_pixel_where::ImageSetPixelWhere;
pub use image_size::ImageSize;
pub use image_sort::{ImageSort, ImageSortMode};
pub use image_split::{ImageSplit, ImageSplitDirection};
pub use image_stack::ImageStack;
pub use image_stats::{ImageStats, ImageStatsMode, Stats};
pub use image_symmetry::ImageSymmetry;
pub use image_tile::ImageTile;
pub use image_to_html::ImageToHTML;
pub use image_to_number::ImageToNumber;
pub use image_trim::ImageTrim;
pub use image_try_create::ImageTryCreate;
pub use image_unicode_formatting::ImageUnicodeFormatting;
pub use inspect_predicted::InspectPredicted;
pub use inspect_task::InspectTask;
pub use label::{ActionLabel, ActionLabelSet, GridLabel, ImageCorner, ImageEdge, ImageLabel, ImageLabelSet, ObjectLabel, ImageProperty, PropertyOutput, SingleColorObjectRectangleLabel, SingleColorObjectSparseLabel, SymmetryLabel};
pub use ngram::{ImageNgram, RecordBigram, RecordTrigram};
pub use number_to_image::NumberToImage;
pub use object_with_different_color::ObjectWithDifferentColor;
pub use object_with_smallest_value::ObjectWithSmallestValue;
pub use objects_and_gravity::{ObjectsAndGravity, ObjectsAndGravityDirection};
pub use objects_and_mass::ObjectsAndMass;
pub use objects_and_position::{ObjectsAndPosition, ObjectsAndPositionMode};
pub use objects_measure_mass::ObjectsMeasureMass;
pub use objects_sort_by_property::ObjectsSortByProperty;
pub use objects_to_grid::{ObjectsToGrid, ObjectsToGridMode};
pub use objects_unique_color_count::ObjectsUniqueColorCount;
pub use output_specification::*;
pub use pixel_connectivity::PixelConnectivity;
pub use popular_objects::PopularObjects;
pub use read_testdata::{path_testdata, read_testdata};
pub use rectangle::Rectangle;
pub use register_arc_functions::register_arc_functions;
pub use reverse_color_popularity::ReverseColorPopularity;
pub use run_with_program::{AnalyzeAndSolve, RunWithProgram, RunWithProgramResult, SolutionSimple, SolutionSimpleData};
pub use shape_identification::{ShapeIdentification, ShapeTransformation, ShapeType};
pub use shape_identification_from_single_color_object::{ShapeIdentificationFromSingleColorObject, ColorAndShape};
pub use single_color_object::{SingleColorObject, SingleColorObjectRectangle, SingleColorObjectSparse, SingleColorObjectCluster, SingleColorObjectClusterContainer};
pub use single_color_object_satisfies_label::SingleColorObjectSatisfiesLabel;
pub use single_color_object_to_label::SingleColorObjectToLabel;
pub use split::{Split, SplitCandidate};
pub use stack_strings::StackStrings;
pub use subcommand_arc_size::SubcommandARCSize;
pub use subcommand_arc_web::SubcommandARCWeb;
pub use substitution_rule::SubstitutionRule;
pub use symmetry::Symmetry;
pub use symmetry_to_label::SymmetryToLabel;
pub use task_graph::*;
pub use time_util::*;
pub use traverse_programs_and_models::TraverseProgramsAndModels;

#[cfg(feature = "linfa")]
mod experiment_with_logisticregression;

#[cfg(feature = "linfa")]
pub use experiment_with_logisticregression::ExperimentWithLogisticRegression;
