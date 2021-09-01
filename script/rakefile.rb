desc 'obtain all the program ids'
file 'data/program_ids.csv' do
    ruby 'task_program_ids.rb'
end

desc 'obtain all the dependencies between programs, for use as input to PageRank algorithm'
file 'data/caller_callee_pairs.csv' => ['data/loda-lab', 'data/program_ids.csv'] do
    ruby 'task_caller_callee_pairs.rb'
end

desc 'obtain all the dependencies between programs, comma separated list'
file 'data/caller_callee_list.csv' => ['data/loda-lab', 'data/program_ids.csv'] do
    ruby 'task_caller_callee_list.rb'
end

desc 'determine the most called programs'
file 'data/most_called_programs.csv' => 'data/caller_callee_list.csv' do
    ruby 'task_most_called_programs.rb'
end

desc 'compute terms with "LODA Rust"'
file 'data/terms_lab.csv' => ['data/loda-lab', 'data/program_ids.csv'] do
    ruby 'task_terms_loda_rust.rb'
end

desc 'compute terms with "LODA Cpp"'
file 'data/terms_loda.csv' => 'data/program_ids.csv' do
    ruby 'task_terms_loda_cpp.rb'
end

desc 'compare terms between "LODA official" and "LODA Lab"'
file 'data/compare_loda_vs_lab.csv' => ['data/terms_lab.csv', 'data/terms_loda.csv'] do
    ruby 'task_compare_loda_vs_lab.rb'
end

desc 'run the PageRank algorithm and ranking the most influential programs'
file 'data/pagerank.csv' => ['data/program_ids.csv', 'data/caller_callee_pairs.csv'] do
    ruby 'task_pagerank.rb'
end

desc 'generate a bigram'
file 'data/bigram.csv' do
    ruby 'task_bigram.rb'
end

desc 'generate a trigram'
file 'data/trigram.csv' do
    ruby 'task_trigram.rb'
end

desc 'generate a skipgram'
file 'data/skipgram.csv' do
    ruby 'task_skipgram.rb'
end

desc 'extract program ids from the LODA denylist file'
file 'data/denylist.csv' do
    ruby 'task_denylist.rb'
end

desc "determine which program ids that shouldn't be attempted mined"
file 'data/dont_mine.csv' => ['data/program_ids.csv', 'data/denylist.csv'] do
    ruby 'task_dont_mine.rb'
end

desc "create a markdown document with the 100 most popular LODA programs"
file 'data/top100.md' => ['data/pagerank.csv', 'data/caller_callee_pairs.csv'] do
    ruby 'task_top100.rb'
end

desc "compiles the loda-lab executable"
file 'data/loda-lab' do
    ruby 'task_lodalab_executable.rb'
end

desc "identify the programs that can be used by the miner"
file 'data/mine_program_ids.csv' => ['data/terms_lab.csv'] do
    ruby 'task_mine_program_ids.rb'
end

desc "extract creation date for all programs"
file 'data/program_creation_dates.csv' do
    ruby 'task_program_creation_dates.rb'
end

desc "extract the most popular programs"
file 'data/program_popularity.csv' => ['data/pagerank.csv'] do
    ruby 'task_program_popularity.rb'
end

task :default do
    system 'rake -T'
end
