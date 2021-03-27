#!/usr/bin/env ruby

=begin

This script traverses all the programs inside the LODA program rootdir.
It looks for all the LODA assembly programs there are.
Each program_id is appended to the CSV file.

This script outputs a `program_ids.csv` file, with this format:

    program id
    4
    5
    7
    8
    10

=end

require 'csv'
require_relative 'config'

LODA_PROGRAM_ROOTDIR = Config.instance.loda_program_rootdir

output_filename = 'data/program_ids.csv'

def obtain_files(path)
    Dir.chdir(path) do
        names = Dir.glob('*').select { |f| File.file? f }
        return names.sort
    end
end

def obtain_dirs(path)
    Dir.chdir(path) do
        names = Dir.glob('*').select { |f| File.directory? f }
        return names.sort
    end
end

def obtain_nested_names(rootdir)
    names = []
    root_dirs = obtain_dirs(rootdir)
    root_dirs.each do |dirname|
        path = File.join(rootdir, dirname)
        names += obtain_files(path)
    end
    names
end

def integers_from_filenames(filenames)
    integers = []
    filenames.map do |filename|
        if filename =~ /0*(\d+)/
            integers << $1.to_i
        else
            puts "Mismatch for #{filename}"
        end
    end
    integers
end


filenames = obtain_nested_names(LODA_PROGRAM_ROOTDIR)
program_ids = integers_from_filenames(filenames)

CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id"]
    program_ids.each_with_index do |program_id, index|
        csv << [program_id.to_s]
    end
end
