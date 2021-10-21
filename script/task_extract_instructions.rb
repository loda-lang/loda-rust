#!/usr/bin/env ruby

=begin
Traverse all the programs there are.
For each program, a corresponding file is outputted.

Extract just the instructions used. Ignoring comments, registers.
So that a program can be boiled down to just a sequence, like this:

    mov
    seq
    pow
    sub

=end

require 'fileutils'
require_relative 'config'

OUTPUT_DIR = 'data/instructions'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
unless File.exist?(LODA_PROGRAMS_OEIS)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

def process_file(path_input, path_output)
    # puts "input: #{path_input}"
    # puts "output: #{path_output}"
    dirname = File.dirname(path_output)
    unless File.directory?(dirname)
        FileUtils.mkdir_p(dirname)
        #puts "creating #{dirname}"
    end
    content = IO.read(path_input)
    content.gsub!(/;.*$/, '')
    content.gsub!(/^\s+/, '')
    content.gsub!(/^(\w+)\b.*$/, '\1')
    content = content.squeeze("\n")
    IO.write(path_output, content)
end

rootdir = LODA_PROGRAMS_OEIS
relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort

approx_count = [relative_paths.count, 10].max
relative_paths.each_with_index do |relative_path, index|
    if (index % 1000) == 0
        percentage = (100 * index) / approx_count
        puts "progress %#{percentage}  #{index}/#{approx_count}"
    end
    # puts "processing: #{relative_path}"
    path_input = File.join(rootdir, relative_path)
    path_output = File.join(OUTPUT_DIR, relative_path)
    path_output.gsub!(/[.]asm$/, '_instructions.txt')
    process_file(path_input, path_output)
end

