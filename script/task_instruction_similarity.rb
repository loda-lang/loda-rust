#!/usr/bin/env ruby

=begin

=end

class Program
    attr_reader :path
    attr_reader :line_count
    
    def initialize(path, line_count)
        @path = path
        @line_count = line_count
    end
end

def split_array_into_clusters(ary, cluster_count)
    result = []
    slice_length = (ary.count + cluster_count - 1) / cluster_count
    ary.each_slice(slice_length) do |slice|
        result << slice
    end
    # pad with empty arrays if needed
    while result.count < cluster_count
        result << []
    end
    result
end

INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

NUMBER_OF_CLUSTERS = 20

unless File.exist?(INPUT_DIR)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

def count_number_of_lines_in_file(path)
    content = IO.read(path)
    content.split("\n").count
end

rootdir = INPUT_DIR
relative_paths = Dir.glob(File.join("**", "*_instructions.txt"), base: rootdir).sort
#relative_paths = relative_paths.first(10)

# Process all the input files and remember the number of lines
program_ary = []
relative_paths.each_with_index do |relative_path, index|
    path_input = File.join(INPUT_DIR, relative_path)
    line_count = count_number_of_lines_in_file(path_input)
    program_ary << Program.new(path_input, line_count)
end
#p program_ary

# Sort by program length.
sorted_program_ary = program_ary.sort { |a,b| a.line_count <=> b.line_count }

# Split into evenly sized clusters
clustered_programs = split_array_into_clusters(sorted_program_ary, NUMBER_OF_CLUSTERS)
clustered_programs.each do |programs|
    p programs.count
end


