desc 'obtain all the program ids'
file 'data/program_ids.csv' do
    ruby 'task_program_ids.rb'
end

desc 'obtain all the dependencies between programs, comma separated list'
file 'data/caller_callee_list.csv' do
    ruby 'task_caller_callee_list.rb'
end

desc 'determine the most called programs'
file 'data/most_called_programs.csv' => 'data/caller_callee_list.csv' do
    ruby 'task_most_called_programs.rb'
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

desc "clean up the inconsistent filenames in the dir for mismatches"
task :cleanup_mismatch_filenames do
    ruby 'task_cleanup_mismatch_filenames.rb'
end

task :default do
    system 'rake -T'
end
