#!/usr/bin/env ruby

=begin

This script takes input from a `pagerank.csv` file, with this format:

    program id;pagerank
    10051;0.0011710868
    10;0.0004871711
    142;0.0004844773
    5;0.0004742994
    40;0.0004690897
    244049;0.0003187378

Determine the popularity group which each program_id belong to.
The miner is more likely to call a popular program_id, eg. the Fibonacci program, or the sqrt2 function.
The miner is less likely to call an unpopular program, eg. an unreferenced program, or few insignificant references. 

This script outputs a `program_popularity.csv` file, with this format:

    program id;popularity
    4;1
    5;9
    6;8
    7;3
    8;0
    10;9
    12;0
    27;0
    30;0
    32;9

=end

require 'csv'

input_filename = 'data/pagerank.csv'
output_filename = 'data/program_popularity.csv'
number_of_groups = 10

class Array
    # split the array in to K equal groups
    # https://stackoverflow.com/a/13634446/78336
    def in_groups(num_groups)
        return [] if num_groups == 0
        slice_size = (self.size/Float(num_groups)).ceil
        groups = self.each_slice(slice_size).to_a
    end
end

# Extract program_id and pagerank
programid_score = []
CSV.foreach(input_filename, col_sep: ";") do |row|
    col0 = row[0]
    col1 = row[1]
    program_id = col0.to_i
    next if program_id == 0
    score = (col1.to_f * 1000000000).to_i
    programid_score << [program_id, score]
end

# Identify the most occuring pagerank
dict = {}
programid_score.each do |program_id, score|
    dict[score] = (dict[score] || 0) + 1
end
found_key = nil
found_value = 0
dict.each do |key, value|
    if value > found_value
        found_key = key
        found_value = value
    end
end
if found_key == nil
    raise "Unable to find the least used program. The programs that has no references are the majority of programs."
end
# p found_key, found_value

score_to_be_ignored = found_key

# Split the program_ids array into two arrays, the popular and the unpopular.
program_ids_ignored = []
program_ids_good = []
programid_score.each do |program_id, score|
    if score == score_to_be_ignored
        program_ids_ignored << program_id
    else
        program_ids_good << program_id
    end
end

# p program_ids.count

program_id_with_group_id = []

# Assign group_id=0 to the unpopular programs, the majority of programs are unreferenced.
program_ids_ignored.each do |program_id|
    program_id_with_group_id << [program_id, 0]
end

# Assign group_id's to the most popular programs
groups_of_program_ids = program_ids_good.in_groups(number_of_groups-1)
groups_of_program_ids.reverse.each_with_index do |program_id_ary, group_index|
    program_id_ary.each do |program_id|
        # puts "#{program_id};#{group_index}"
        # puts "#{group_index}"
        program_id_with_group_id << [program_id, group_index+1]
    end
end

# Sort by program_id, so that the file can be checked into git and preserves some of its structure across commits.
program_id_with_group_id.sort_by! { |program_id,group_index| program_id }

# Create csv file with result
CSV.open(output_filename, "wb", col_sep: ";") do |csv|
    csv << ["program id", "popularity"]
    program_id_with_group_id.each do |program_id, group_id|
        csv << [program_id, group_id]
    end
end


# Print a status report
dict = {}
program_id_with_group_id.each do |program_id,group_id|
    dict[group_id] = (dict[group_id] || 0) + 1
end
ary = []
dict.keys.sort.each do |key|
    count = dict[key]
    ary << "#{key}=>#{count}"
end
puts "number of programs in each cluster: #{ary.join(', ')}" 
# 0=>25444, 1=>301, 2=>307, 3=>307, 4=>307, 5=>307, 6=>307, 7=>307, 8=>307, 9=>307
