#!/usr/bin/env ruby

=begin

=end

require 'csv'
require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
unless File.exist?(LODA_PROGRAMS_OEIS)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
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

def process_program_file(path)
    path =~ /\bA0*(\d+)[.]asm$/
    program_id = $1.to_i
    if program_id == 0
        puts "Mismatch for #{path}"
        return
    end
    puts "match: #{program_id}"
    content = IO.read(path)
end

paths = absolute_paths_for_unstaged_programs
#p paths
if paths.empty?
    puts "There are no unstaged .asm files in this repository."
    exit 0
end

puts "Number of unstaged .asm files: #{paths.count}"

paths.each do |path|
    process_program_file(path)
end

