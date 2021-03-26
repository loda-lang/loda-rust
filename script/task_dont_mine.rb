#!/usr/bin/env ruby

=begin

Determine which program ids that shouldn't be attempted mined. 
Either because they already have been mined succesfully, or because they are an OEIS duplicate, or junk data.

This script takes input from the `denylist.csv` file, with this format:

    program id
    17
    154
    381
    572

This script takes input from a `program_ids.csv` file, with this format:

    program id
    4
    5
    7
    8
    10

The goal is the create programs for all OEIS sequences.
This script determines what OEIS sequences that should not be inserted into the bloom filter.
This way the miner will not attempt to mine the sequence.

Sequences with junk that doesn't make sense to create programs for.
It's wasteful trying to fit a formula to these. These can safely be ignored.

It's wasteful to create programs for sequences that already have an existing program, that is highly optimized.
As of 24mar2021, there is no way to distinguish between the optimization levels.
So for now, the sequences that have already has a corresponding LODA program, gets ignored.
Maybe in the future, the inefficient programs should be added, for finding more efficient versions.

This script outputs a `dont_mine.csv` file, with this format:

    program id
    17
    154
    381
    572

=end

require 'csv'
require 'set'

input_filename0 = 'data/denylist.csv'
input_filename1 = 'data/program_ids.csv'
output_filename = 'data/dont_mine.csv'

program_id_set = Set.new
CSV.foreach(input_filename0, {:col_sep => ";"}) do |row|
    col0 = row[0]
    program_id = col0.to_i
    next if program_id == 0
    program_id_set << program_id
end
CSV.foreach(input_filename1, {:col_sep => ";"}) do |row|
    col0 = row[0]
    program_id = col0.to_i
    next if program_id == 0
    program_id_set << program_id
end

additional_ignore = [
    109811,  # majority of false positives
    221222,  # majority of false positives
    105360,  # some false positives
]
program_id_set += additional_ignore

program_ids = program_id_set.to_a.sort
puts "number of program ids: #{program_ids.count}"

# Generate output file
CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id"]
    program_ids.each_with_index do |program_id, index|
        csv << [program_id.to_s]
    end
end
