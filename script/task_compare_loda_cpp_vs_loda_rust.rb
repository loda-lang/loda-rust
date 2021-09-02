#!/usr/bin/env ruby

=begin

This script takes 2 input files with this format:

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

This script compares terms of all rows.
If the terms are the same it says "ok".
If the terms are different it says "mismatch".

This script outputs a `compare_loda_cpp_vs_loda_rust.csv` file, with this format:

    program id;status;actual;expected
    5;ok;1,2,2,3,2,4,2,4,3,4;1,2,2,3,2,4,2,4,3,4
    5492;mismatch;BOOM;4,15,52,151,372,799,1540,2727,4516,7087
    52558;mismatch;1,1,4,12,72,360,2880,20160,201600,1814400;BOOM
    66143;mismatch;1,3,8,18,44,150,762,5096,40392,362970;BOOM
    70747;ok;0,1,1,1,-1,-1,-1,1,1,1;0,1,1,1,-1,-1,-1,1,1,1

=end

require 'csv'

def load_terms_into_dict(input_filename)
    dict = {}
    CSV.foreach(input_filename, {:col_sep => ";"}) do |col0, col1|
        program_id = col0.to_i
        terms = col1
        next if program_id == 0
        dict[program_id] = terms
    end
    dict
end

input_filename0 = 'data/terms_loda_rust.csv'
input_filename1 = 'data/terms_loda.csv'
output_filename = 'data/compare_loda_cpp_vs_loda_rust.csv'
time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)


dict0 = load_terms_into_dict(input_filename0)
dict1 = load_terms_into_dict(input_filename1)
# p dict0.count
# p dict1.count

program_ids = (dict0.keys + dict1.keys).uniq.sort
# p program_ids.count

# Status 
count_success = 0
count_failure = 0
program_ids_count_minus1 = program_ids.count - 1
if program_ids_count_minus1 == 0
    program_ids_count_minus1 = 1
end

# Generate output file
CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id", "status", "actual", "expected"]
    program_ids.each_with_index do |program_id, index|
        actual = dict0[program_id]
        expected = dict1[program_id]
        success = actual == expected
        if success
            count_success += 1
            csv << [program_id.to_s, "ok", actual, expected]
        else
            count_failure += 1
            csv << [program_id.to_s, "mismatch", actual, expected]
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
