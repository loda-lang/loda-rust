#!/usr/bin/env ruby

=begin

This script takes input from the `denylist.txt` file, with this format:

    A000017: Erroneous version of A032522.
    A000154: Erroneous version of A003713.
    A000381: Essentially the same as A001611.
    A000572: A Beatty sequence: [ n(e+1) ].

This script outputs a `denylist.csv` file, with this format:

    program id
    17
    154
    381
    572

=end

require 'csv'
require 'dotenv'
Dotenv.load('../.env')

LODA_PROGRAM_ROOTDIR = ENV['LODA_PROGRAM_ROOTDIR']

input_filename = File.join(LODA_PROGRAM_ROOTDIR, 'denylist.txt')
output_filename = 'data/denylist.csv'

program_ids = []
File.open(input_filename, 'r') do |file|
    file.each_line do |line|
        line =~ /^A(\d+):/
        program_id = $1.to_i
        next if program_id == 0
        program_ids << program_id
    end
end
puts "number of program ids: #{program_ids.count}"

# Generate output file
CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id"]
    program_ids.each_with_index do |program_id, index|
        csv << [program_id.to_s]
    end
end
