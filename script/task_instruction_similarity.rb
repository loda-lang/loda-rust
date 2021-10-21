#!/usr/bin/env ruby

=begin

=end

INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

NUMBER_OF_CLUSTERS = 40
PERCENTAGE_MUST_BE_IDENTICAL = 0.8

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

def compare_programs(current_program, program_of_similar_length)
    line_count0 = current_program.line_count
    line_count1 = program_of_similar_length.line_count
    lc_diff = (line_count0 - line_count1).abs
    if line_count0 < 5 || line_count1 < 5
        if lc_diff > 0
            # puts "skip 0"
            return
        end
    else
        if line_count0 < 10 && line_count1 < 10
            if lc_diff > 4
                # puts "skip 1"
                return
            end
        end
    end
    path0 = current_program.path
    path1 = program_of_similar_length.path
    cmd = "diff --unchanged-group-format='%<' --old-group-format='' --new-group-format='' #{path0} #{path1}"
    #puts "will execute: #{cmd}"
    output = `#{cmd}`
    output.strip!
    if output.empty?
        # puts "skip 2"
        return
    end
    number_of_identical_lines = output.split("\n").count
    target0 = (line_count0 * PERCENTAGE_MUST_BE_IDENTICAL).ceil
    target1 = (line_count1 * PERCENTAGE_MUST_BE_IDENTICAL).ceil
    if number_of_identical_lines < target0
        # puts "skip 3 #{number_of_identical_lines} < #{target0}"
        return
    end
    if number_of_identical_lines < target1
        # puts "skip 4 #{number_of_identical_lines} < #{target1}"
        return
    end
    puts "similar #{current_program.path} with #{program_of_similar_length.path}. number of lines shared: #{number_of_identical_lines}"
end

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
sorted_program_ary = sorted_program_ary.reverse

# Split into evenly sized clusters
clustered_programs = split_array_into_clusters(sorted_program_ary, NUMBER_OF_CLUSTERS)

# Compare all programs inside the same cluster with each other
program_index = 0
clustered_programs.each_with_index do |programs_of_similar_length, cluster_index|
    programs_of_similar_length.each_with_index do |current_program, current_program_index|
        next if current_program.line_count < 10
        next if current_program.line_count > 30
        programs_of_similar_length.each do |program_of_similar_length|
            if current_program === program_of_similar_length
                next
            end
            if (program_index % 10000) == 0
                puts "program: #{program_index} cluster: #{cluster_index} current_program_index: #{current_program_index}"
            end
            compare_programs(current_program, program_of_similar_length)
            program_index += 1
        end
    end
end


