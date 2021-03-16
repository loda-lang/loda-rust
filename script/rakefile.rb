desc 'obtain all the program ids'
file 'data/program_ids.csv' do
    ruby 'task_program_ids.rb'
end

desc 'obtain all the dependencies between programs, for use as input to PageRank algorithm'
file 'data/caller_callee_pairs.csv' => 'data/program_ids.csv' do
    ruby 'task_caller_callee_pairs.rb'
end

desc 'obtain all the dependencies between programs, comma separated list'
file 'data/caller_callee_list.csv' => 'data/program_ids.csv' do
    ruby 'task_caller_callee_list.rb'
end

desc 'determine the most called programs'
file 'data/most_called_programs.csv' => 'data/caller_callee_list.csv' do
    ruby 'task_most_called_programs.rb'
end

desc 'compute terms with "LODA Lab"'
file 'data/terms_lab.csv' => 'data/program_ids.csv' do
    ruby 'task_terms_lab.rb'
end

desc 'compute terms with "LODA Official"'
file 'data/terms_loda.csv' => 'data/program_ids.csv' do
    ruby 'task_terms_loda.rb'
end

desc 'compare terms between "LODA official" and "LODA Lab"'
file 'data/compare_loda_vs_lab.csv' => ['data/program_ids.csv', 'data/terms_loda.csv'] do
    ruby 'task_compare_loda_vs_lab.rb'
end

desc 'run the PageRank algorithm and ranking the most influential programs'
file 'data/pagerank.csv' => ['data/program_ids.csv', 'data/caller_callee_pairs.csv'] do
    ruby 'task_pagerank.rb'
end

task :default do
    system 'rake -T'
end
