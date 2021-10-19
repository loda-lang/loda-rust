#!/usr/bin/env ruby

=begin

=end

require 'set'
require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
unless File.exist?(LODA_PROGRAMS_OEIS)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

OEIS_NAMES_FILE = Config.instance.oeis_names_file
unless File.exist?(OEIS_NAMES_FILE)
    raise "No such file #{OEIS_NAMES_FILE}, cannot run script"
end

# git: list new files only
# https://stackoverflow.com/a/26891150/78336
def absolute_paths_for_unstaged_files(repo_rootdir)
    paths1 = []
    Dir.chdir(repo_rootdir) do
        result = `git ls-files -o  --exclude-standard`
        paths1 = result.split(/\n/)
    end
    paths2 = paths1.map do |path|
        File.join(repo_rootdir, path)
    end
    paths2
end

def absolute_paths_for_unstaged_programs
    paths1 = absolute_paths_for_unstaged_files(LODA_PROGRAMS_OEIS)
    paths2 = paths1.filter { |path| path =~ /[.]asm$/ }
    paths3 = paths2.filter { |path| path =~ /\boeis\b/ }
    paths3
end

def extract_oeis_ids_from_program_file(path)
    path =~ /\bA0*(\d+)[.]asm$/
    program_id = $1.to_i
    if program_id == 0
        raise "Unable to process file at path: #{path}"
    end
    content = IO.read(path)
    sequence_instructions = content.scan(/^\s*seq .*,\s*(\d+)$/).flatten
    sequence_program_ids = sequence_instructions.map { |seq_program_id| seq_program_id.to_i }
    # puts "program: #{program_id} depends on: #{sequence_program_ids}"
    return [program_id] + sequence_program_ids
end

def extract_oeis_ids_from_program_files(paths)
    program_ids = []
    paths.each do |path|
        program_ids += extract_oeis_ids_from_program_file(path)
    end
    program_ids
end

paths = absolute_paths_for_unstaged_programs
#p paths
if paths.empty?
    puts "There are no unstaged .asm files in this repository."
    exit 0
end

puts "Number of unstaged .asm files: #{paths.count}"

# Harvest all the program ids that needs to have their name looked up
program_ids = extract_oeis_ids_from_program_files(paths)
program_ids_set = program_ids.to_set
puts "Will lookup names for these program ids: #{program_ids_set.to_a.sort}" 

approx_row_count = 350000
File.new(OEIS_NAMES_FILE, "r").each_with_index do |line, index|
    if (index % 30000) == 0
        percentage = (100 * index) / approx_row_count
        puts "progress %#{percentage}  #{index}/#{approx_row_count}"
    end
    next unless line =~ /^A0*(\d+) (.+)$/
    program_id = $1.to_i
    name = $2
    if program_ids_set.include?(program_id)
        puts "program_id: #{program_id} name: #{name}"
    end
end
