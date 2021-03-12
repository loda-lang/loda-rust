#!/usr/bin/env ruby

=begin
Prerequisits:
The 'dotenv' gem. Install it using `$ gem install dotenv`
https://github.com/bkeepers/dotenv

This script traverses all the programs inside the LODA program rootdir.
It does search-and-replace through all the LODA assembly programs there are.
When encountering a program that contains a `lpb $x,1` instruction, then it's being replaced by `lpb $x`.

=end

require 'csv'
require 'dotenv'
Dotenv.load('../.env')

LODA_PROGRAM_ROOTDIR = ENV['LODA_PROGRAM_ROOTDIR']

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def process_paths(paths)
    paths.map do |path|
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        content = IO.read(path)
        n = content.scan(/lpb [$]\d+,1/).count
        if n == 0
            next
        end
        content.gsub!(/(lpb [$]\d+),1\b/, '\1')
        IO.write(path, content)
    end
end

paths = absolute_paths_for_all_programs(LODA_PROGRAM_ROOTDIR)
process_paths(paths)
