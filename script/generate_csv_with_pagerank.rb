#!/usr/bin/env ruby

=begin

This script sorts the rows by their pagerank (scientific notation).

This script takes input from a `pagerank_input.csv` file, with this format:

    program id;pagerank
    4;3.69424213419305e-06
    5;0.000474322958227094
    7;4.59141509988159e-06
    10;0.000487195266031759
    32;3.78869376642486e-05
    40;0.000469112703118165

This script outputs a `pagerank_output.csv` file, with this format:

    program id;pagerank
    10051;0.0011711603
    10;0.0004871953
    142;0.0004845017
    5;0.0004743230
    40;0.0004691127
    244049;0.0003187596
    230980;0.0002805656
    20639;0.0002301204

=end

require 'csv'

input_filename = 'pagerank_input.csv'
output_filename = 'pagerank_output.csv'

rows = []

puts "input: #{input_filename}"
count = 0
CSV.foreach(input_filename, {:col_sep => ";"}) do |col0, col1|
    program_id = col0.to_i
    pagerank = col1.to_f
    rows << [pagerank, program_id]
    count += 1
    # break if count > 10
end
rows = rows.sort.reverse

puts "output: #{output_filename}"
CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id", "pagerank"]

    rows.each do |pagerank, program_id|
        pretty_pagerank = "%0.10f" % pagerank
        csv << [program_id, pretty_pagerank]
    end
end

