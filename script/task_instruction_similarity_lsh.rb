#!/usr/bin/env ruby

=begin
My second attempt at identifying similar programs.
=end

require 'csv'
require 'set'

INPUT_FILE_BIGRAM = 'data/bigram.csv'
INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

NUMBER_OF_CLUSTERS = 40
PERCENTAGE_MUST_BE_IDENTICAL = 0.8

unless File.exist?(INPUT_DIR)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

unless File.exist?(INPUT_FILE_BIGRAM)
    raise "No such file #{INPUT_FILE_BIGRAM}, cannot run script"
end

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


def count_number_of_lines_in_file(path)
    content = IO.read(path)
    content.split("\n").count
end


def load_bigram(path)
    vocabulary = []
    CSV.foreach(path, col_sep: ";") do |row|
        col0 = row[0]
        count = col0.to_i
        next if count == 0
        word0 = row[1]
        word1 = row[2]
        vocabulary << "#{word0}#{word1}"
    end
    vocabulary.sort
end

vocabulary = load_bigram(INPUT_FILE_BIGRAM)
p vocabulary
raise "x"

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
    output_name = relative_path.gsub('_instructions.txt', '_similarity2.csv')
    path_output = File.join(OUTPUT_DIR, output_name)
    # line_count = count_number_of_lines_in_file(path_input)
    # program_ary << Program.new(program_id, path_input, path_output, line_count)
end
#p program_ary
