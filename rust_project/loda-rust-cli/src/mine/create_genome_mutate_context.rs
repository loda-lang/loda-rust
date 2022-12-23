use crate::config::Config;
use crate::analytics::AnalyticsDirectory;
use super::{GenomeMutateContext, GenomeMutateContextBuilder};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CreateGenomeMutateContextMode {
    OEIS,
    ARC,
}

pub fn create_genome_mutate_context(mode: CreateGenomeMutateContextMode, config: &Config, analytics_directory: AnalyticsDirectory) -> anyhow::Result<GenomeMutateContext> {
    let loda_rust_repository: PathBuf = config.loda_rust_repository();
    let recent_program_csv = loda_rust_repository.join(Path::new("resources/program_creation_dates.csv"));

    let instruction_trigram_csv: PathBuf = analytics_directory.histogram_instruction_trigram_file();
    let line_trigram_csv: PathBuf = analytics_directory.histogram_line_trigram_file();
    let source_trigram_csv: PathBuf = analytics_directory.histogram_source_trigram_file();
    let target_trigram_csv: PathBuf = analytics_directory.histogram_target_trigram_file();
    let histogram_instruction_constant_csv: PathBuf = analytics_directory.histogram_instruction_constant_file();
    let popular_program_csv: PathBuf = analytics_directory.program_popularity_file();
    let valid_program_csv: PathBuf = analytics_directory.programs_valid_file();
    let invalid_program_csv: PathBuf = analytics_directory.programs_invalid_file();
    let indirect_memory_access_csv: PathBuf = analytics_directory.indirect_memory_access_file();

    let mut builder = GenomeMutateContextBuilder::new();
    builder.suggest_instruction(&instruction_trigram_csv)?;
    builder.suggest_line(&line_trigram_csv)?;
    builder.suggest_source(&source_trigram_csv)?;
    builder.suggest_target(&target_trigram_csv)?;
    builder.histogram_instruction_constant(&histogram_instruction_constant_csv)?;

    if mode == CreateGenomeMutateContextMode::OEIS {
        builder.recent_programs(&recent_program_csv)?;
        builder.popular_programs(&popular_program_csv)?;
        builder.valid_programs(&valid_program_csv)?;
        builder.invalid_programs(&invalid_program_csv)?;
        builder.indirect_memory_access_program_ids(&indirect_memory_access_csv)?;
    }

    let context: GenomeMutateContext = builder.build()?;
    Ok(context)
}
