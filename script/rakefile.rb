desc 'obtain all the program ids'
file 'data/program_ids.csv' do
    ruby 'task_program_ids.rb'
end

desc 'compute terms with "loda-rust"'
file 'data/terms_loda_rust.csv' => ['data/loda-rust', 'data/program_ids.csv'] do
    ruby 'task_terms_loda_rust.rb'
end

desc 'compute terms with "loda-cpp"'
file 'data/terms_loda_cpp.csv' => 'data/program_ids.csv' do
    ruby 'task_terms_loda_cpp.rb'
end

desc 'compare terms between "loda-cpp" and "loda-rust"'
file 'data/compare_loda_cpp_vs_loda_rust.csv' => ['data/terms_loda_rust.csv', 'data/terms_loda_cpp.csv'] do
    ruby 'task_compare_loda_cpp_vs_loda_rust.rb'
end

desc "create a markdown document with the 100 most popular LODA programs"
file 'data/top100.md' do
    ruby 'task_top100.rb'
end

desc "compiles the loda-rust executable"
file 'data/loda-rust' do
    ruby 'task_loda_rust_executable.rb'
end

desc "extract creation date for all programs"
file 'data/program_creation_dates.csv' do
    ruby 'task_program_creation_dates.rb'
end

desc "Process the mined programs"
task :process_mined_programs => 'data/loda-rust' do
    ruby 'task_analyze_mined_programs.rb'
    ruby 'insert_oeis_names_into_program.rb'
    ruby 'task_add_mined_programs_to_repo.rb'
    ruby 'task_cleanup_temp_files_from_repo.rb'
    ruby 'task_maintenance_of_outlier_programs_repo.rb'
end

desc "Remove already processed programs with suffix .keep.asm and .reject.asm"
task :clean_mineevent_dir do
    ruby 'task_cleanup_processed_files_from_mineevent_dir.rb'
end

task :default do
    system 'rake -T'
end
