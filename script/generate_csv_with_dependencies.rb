#!/usr/bin/env ruby

=begin
Prerequisits:
The 'dotenv' gem. Install it using `$ gem install dotenv`
https://github.com/bkeepers/dotenv



This script traverses all the programs inside the LODA program rootdir.
Each program is a line in the CSV file, with it's dependencies listed.

This script outputs a `dependenies.csv` file, with this format:

    program id;dependency count;program ids
    4;1;4
    5;1;5
    7;1;7
    8;2;8,165190
    10;1;10

=end

require 'csv'
require 'dotenv'
Dotenv.load('../.env')

LODA_PROGRAM_ROOTDIR = ENV['LODA_PROGRAM_ROOTDIR']

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

def sequenceids_from_filenames(filenames)
    sequence_ids = []
    filenames.map do |filename|
        if filename =~ /0*(\d+)/
            sequence_id = $1.to_i
            sequence_ids << sequence_id
        else
            puts "Mismatch for #{filename}"
        end
    end
    sequence_ids
end


# Build the newest version
`cargo build --release`

time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

filenames = obtain_nested_names(LODA_PROGRAM_ROOTDIR)
sequence_ids = sequenceids_from_filenames(filenames)
# p sequence_ids

count_success = 0
count_failure = 0
sequence_ids_count_minus1 = sequence_ids.count - 1
if sequence_ids_count_minus1 == 0
    sequence_ids_count_minus1 = 1
end

CSV.open("dependencies.csv", "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id", "dependency count", "program ids"]
    sequence_ids.each_with_index do |sequence_id, index|
        output = `../target/release/loda_lab dependencies #{sequence_id}`
        output = output.strip
        dependency_count = output.split(',').count
        success = $?.success?
        if success
            count_success += 1
            csv << [sequence_id.to_s, dependency_count.to_s, output]
        else
            count_failure += 1
        end
        if (index % 1000) == 0
            percent = (100 * index) / sequence_ids_count_minus1
            puts "PROGRESS: #{index} / #{sequence_ids.count}  percent: #{percent}"
        end
        # if index == 10
        #     break
        # end
    end
end

time_end = Process.clock_gettime(Process::CLOCK_MONOTONIC)
time_elapsed = time_end - time_start
time_elapsed_s = "%.3f" % time_elapsed
puts "elapsed: #{time_elapsed_s}"

puts "count_success: #{count_success}"
puts "count_failure: #{count_failure}"