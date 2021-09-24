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
Each program is a line in the CSV file, with it's dependencies listed.

This script outputs a `caller_callee_pairs.csv` file, with this format:

    caller program id;callee program id
    73;232508
    73;301657
    134;22844
    134;121381
    134;246388
    134;4082

=end

require 'csv'

input_filename = 'data/program_ids.csv'
output_filename = 'data/caller_callee_pairs.csv'

time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

# Obtain all the program_ids to be processed
program_ids = []
CSV.foreach(input_filename, col_sep: ";") do |row|
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
CSV.open(output_filename, "wb", col_sep: ";") do |csv|
    csv << ["caller program id", "callee program id"]
    program_ids.each_with_index do |program_id, index|
        output = `data/loda-rust dependencies #{program_id}`
        output = output.strip
        success = $?.success?
        if success
            count_success += 1
            dependency_array = output.split(',')
            dependency_array.drop(1).each do |callee_program_id|
                csv << [program_id.to_s, callee_program_id]
            end
        else
            count_failure += 1
        end
        if (index % 1000) == 0
            percent = (100 * index) / program_ids_count_minus1
            puts "PROGRESS: #{index} / #{program_ids.count}  percent: #{percent}"
        end
        # break if index == 10
    end
end

# Show stats
time_end = Process.clock_gettime(Process::CLOCK_MONOTONIC)
time_elapsed = time_end - time_start
time_elapsed_s = "%.3f" % time_elapsed
puts "elapsed: #{time_elapsed_s}"

puts "count_success: #{count_success}"
puts "count_failure: #{count_failure}"