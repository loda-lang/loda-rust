#!/usr/bin/env ruby

=begin

Mining is computationally expensive. 

The purpose of this script is to make mining less expensive,
by identifying defunct programs, so that it doesn't happen at runtime.

This script takes input from a `terms_lab.csv` file, with this format:

    program id;terms
    4;0,0,0,0,0,0,0,0,0,0
    5;1,2,2,3,2,4,2,4,3,4
    7;1,0,0,0,0,0,0,0,0,0
    8;1,1,2,2,3,4,5,6,7,8
    10;1,1,2,2,4,2,6,4,6,4
    12;BOOM
    27;1,2,3,4,5,6,7,8,9,10
    30;0,1,2,3,4,5,6,7,8,9
    32;2,1,3,4,7,11,18,29,47,76

If there is a row that contains `BOOM` it means that there
was a problem with the program. Usually there is a cyclic dependency, 
a missing dependency, or the program divides by zero.
If there are 10 terms, then the program appears to be runnable.

This script outputs a `mine_program_ids.csv` file, with this format:

    program id
    4
    5
    7
    8
    10

=end

require 'csv'
require 'set'

def extract_good_program_ids(input_filename)
    result = Set.new
    CSV.foreach(input_filename, {:col_sep => ";"}) do |col0, col1|
        program_id = col0.to_i
        terms = col1
        next if program_id == 0
        next if terms !~ /\d,\d+,\d/
        result.add(program_id)
    end
    result.to_a.sort
end

input_filename = 'data/terms_lab.csv'
output_filename = 'data/mine_program_ids.csv'

program_ids = extract_good_program_ids(input_filename)
#p program_ids.count

CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id"]
    program_ids.each_with_index do |program_id, index|
        csv << [program_id.to_s]
    end
end
