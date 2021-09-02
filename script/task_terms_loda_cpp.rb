#!/usr/bin/env ruby

=begin

This script takes input from a `program_ids.csv` file, with this format:

    program id
    4
    5
    7
    8
    10

This script traverses all the programs inside the LODA program rootdir.
Each program is evaluated and the outputs is stored in the CSV file.

This script outputs a `terms_loda_cpp.csv` file, with this format:

    program id;terms
    4;0,0,0,0,0,0,0,0,0,0
    5;1,2,2,3,2,4,2,4,3,4
    7;1,0,0,0,0,0,0,0,0,0
    8;1,1,2,2,3,4,5,6,7,8
    10;1,1,2,2,4,2,6,4,6,4
    12;1,1,1,1,1,1,1,1,1,1
    27;1,2,3,4,5,6,7,8,9,10
    30;0,1,2,3,4,5,6,7,8,9
    32;2,1,3,4,7,11,18,29,47,76

=end

require 'csv'
require_relative 'config'

LODA_CPP_REPOSITORY = Config.instance.loda_cpp_repository

input_filename = 'data/program_ids.csv'
output_filename = 'data/terms_loda_cpp.csv'
time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

# Obtain all the program_ids to be processed
program_ids = []
CSV.foreach(input_filename, {:col_sep => ";"}) do |row|
    col0 = row[0]
    program_id = col0.to_i
    next if program_id == 0
    program_ids << program_id
end

# Status 
count_success = 0
count_failure = 0
program_ids_count_minus1 = program_ids.count - 1
if program_ids_count_minus1 == 0
    program_ids_count_minus1 = 1
end

# Generate output file
absolute_path_to_output_file = File.join(Dir.pwd, output_filename)
Dir.chdir(LODA_CPP_REPOSITORY) do
    CSV.open(absolute_path_to_output_file, "wb", {:col_sep => ";"}) do |csv|
        csv << ["program id", "terms"]
        program_ids.each_with_index do |program_id, index|
            output = `./loda eval A#{program_id} -t 10`
            output = output.strip
            success = $?.success?
            if success
                count_success += 1
                csv << [program_id.to_s, output]
            else
                count_failure += 1
                csv << [program_id.to_s, "BOOM"]
            end
            if (index % 1000) == 0
                percent = (100 * index) / program_ids_count_minus1
                puts "PROGRESS: #{index} / #{program_ids.count}  percent: #{percent}"
            end
            # break if index == 10
        end
    end
end

# Show stats
time_end = Process.clock_gettime(Process::CLOCK_MONOTONIC)
time_elapsed = time_end - time_start
time_elapsed_s = "%.3f" % time_elapsed
puts "elapsed: #{time_elapsed_s}"

puts "count_success: #{count_success}"
puts "count_failure: #{count_failure}"
