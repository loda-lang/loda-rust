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

INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

unless File.exist?(INPUT_DIR)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

def count_number_of_lines_in_file(path)
    content = IO.read(path)
    content.split("\n").count
end

rootdir = INPUT_DIR
relative_paths = Dir.glob(File.join("**", "*_instructions.txt"), base: rootdir).sort
relative_paths = relative_paths.first(10)
#p relative_paths.first(10)

program_ary = []
relative_paths.each_with_index do |relative_path, index|
    path_input = File.join(INPUT_DIR, relative_path)
    line_count = count_number_of_lines_in_file(path_input)
    program_ary << Program.new(path_input, line_count)
end
p program_ary








