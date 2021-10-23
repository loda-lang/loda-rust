#!/usr/bin/env ruby

=begin
My first attempt at identifying similar programs. It's terrible slow.
It uses the `diff` command for counting the number of identical lines shared between two files.

There are 41000 programs in the "loda-programs" repo at the moment.
Invoking diff between all programs would require NxN operations.
In order to avoid that, the programs are grouped into 40 different clusters.
Short programs go together in the cluster for short programs.
Long programs go into the cluster for long programs.
So 1 program only have to be compared with 1025 (41000 / 40) other programs.

For every program, a CSV file is outputted, like this:

A014368_similarity.csv
program_id;overlap_count;jaccard_index
155096;16;0.6957
255995;16;0.6400

The above file means that A014368 is similar to A155096 
and that A014368 is also similar to A255995.

If 80% of the lines are identical, then the program gets added to the CSV file.

=end

require 'csv'

INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

NUMBER_OF_CLUSTERS = 40
PERCENTAGE_MUST_BE_IDENTICAL = 0.8

class Program
    attr_reader :program_id
    attr_reader :path_input
    attr_reader :path_output
    attr_reader :line_count
    
    def initialize(program_id, path_input, path_output, line_count)
        @program_id = program_id
        @path_input = path_input
        @path_output = path_output
        @line_count = line_count
    end
end

class ComparisonResult
    attr_reader :program_id
    attr_reader :a_line_count
    attr_reader :b_line_count
    attr_reader :overlap_line_count
    
    def initialize(program_id, a_line_count, b_line_count, overlap_line_count)
        @program_id = program_id
        @a_line_count = a_line_count
        @b_line_count = b_line_count
        @overlap_line_count = overlap_line_count
    end
    
    def jaccard_index
        x = @overlap_line_count
        y = @a_line_count + @b_line_count - @overlap_line_count
        x.to_f / y.to_f
    end
    
    def human_readable_jaccard_index
        "%.4f" % jaccard_index
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

def compare_programs(program0, program1)
    line_count0 = program0.line_count
    line_count1 = program1.line_count
    lc_diff = (line_count0 - line_count1).abs
    if line_count0 < 5 || line_count1 < 5
        if lc_diff > 0
            # puts "skip 0"
            return nil
        end
    else
        if line_count0 < 10 && line_count1 < 10
            if lc_diff > 4
                # puts "skip 1"
                return nil
            end
        end
    end
    path0 = program0.path_input
    path1 = program1.path_input
    cmd = "diff --unchanged-group-format='%<' --old-group-format='' --new-group-format='' #{path0} #{path1}"
    #puts "will execute: #{cmd}"
    output = `#{cmd}`
    output.strip!
    if output.empty?
        # puts "skip 2"
        return nil
    end
    number_of_identical_lines = output.split("\n").count
    target = (line_count0 * PERCENTAGE_MUST_BE_IDENTICAL).ceil
    if number_of_identical_lines < target
        # puts "skip 3 #{number_of_identical_lines} < #{target}"
        return nil
    end
    puts "similar #{program0.program_id} with #{program1.program_id}. number of lines shared: #{number_of_identical_lines}"
    return ComparisonResult.new(program1.program_id, line_count0, line_count1, number_of_identical_lines)
end

def save_similar_programs(current_program, comparison_result_array)
    path = current_program.path_output
    CSV.open(path, "wb", col_sep: ";") do |csv|
        csv << ["program_id", "overlap_count", "jaccard_index"]
        comparison_result_array.each_with_index do |comparison_result, index|
            row = [
                comparison_result.program_id,
                comparison_result.overlap_line_count,
                comparison_result.human_readable_jaccard_index
            ]
            csv << row
            # break if index == 10
        end
    end
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
    relative_path =~ /\bA0*(\d+)_/
    program_id = $1.to_i
    if program_id == 0
        puts "Ignoring invalid program id for relative_path: #{relative_path}"
        next
    end
    path_input = File.join(INPUT_DIR, relative_path)
    output_name = relative_path.gsub('_instructions.txt', '_similarity.csv')
    path_output = File.join(OUTPUT_DIR, output_name)
    line_count = count_number_of_lines_in_file(path_input)
    program_ary << Program.new(program_id, path_input, path_output, line_count)
end
#p program_ary

# Sort by program length.
sorted_program_ary = program_ary.sort { |a,b| a.line_count <=> b.line_count }
sorted_program_ary = sorted_program_ary.reverse

# Split into evenly sized clusters
clustered_programs = split_array_into_clusters(sorted_program_ary, NUMBER_OF_CLUSTERS)

# Compare all programs inside the same cluster with each other
program_index = 0
program_count = [sorted_program_ary.count, 10].max
clustered_programs.each_with_index do |programs_of_similar_length, cluster_index|
    programs_of_similar_length.each_with_index do |current_program, current_program_index|
        next if current_program.line_count < 10
        next if current_program.line_count > 30
        
        comparison_result_array = []
        programs_of_similar_length.each do |program_of_similar_length|
            if current_program === program_of_similar_length
                next
            end
            comparison_result = compare_programs(current_program, program_of_similar_length)
            if comparison_result != nil
                comparison_result_array << comparison_result
            end
        end
        comparison_result_array.sort! { |a,b| a.program_id <=> b.program_id }
        
        save_similar_programs(current_program, comparison_result_array)
        
        percent = program_index * 100 / program_count
        progress = "#{percent}\% #{program_index}/#{program_count}" 
        puts "#{progress}  #{current_program.program_id} is similar to #{comparison_result_array.count} other programs."
        program_index += 1
    end
end


