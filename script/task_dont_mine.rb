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

# These are causing most of the false positives
additional_ignore = [
    38126,  # a(n) = floor( sqrt(2*Pi)*n ) (a Beatty sequence).
    68670,  # Number of digits in the concatenation of first n primes.
    105360, # Records in A105358.
    109811, # Triangular numbers (A000217) at Levenshtein distance 1 from another triangular number when considered as a decimal string.
    172337, # Floor(n*(sqrt(11)+sqrt(7))).
    172338, # a(n) = floor(n*(sqrt(5)+sqrt(3))).
    183140, # a(n) = [1/s]+[2/s]+...+[n/s], where s=2+sqrt(2) and []=floor.
    221222, # Threshold for the P(n)-avoidance vertex-coloring game
    328588, # Numbers n for which A257993(A276086(A276086(n))) is larger than A257993(n)
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
