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

This script takes input from a `caller_callee_pairs.csv` file, with this format:

    caller program id;callee program id
    73;232508
    73;301657
    134;22844
    134;121381
    134;246388
    134;4082

This script outputs a markdown document.
The document shows a human readable table of the 100 most popular LODA programs.

=end

require 'csv'

input_filename0 = 'data/pagerank.csv'
input_filename1 = 'data/caller_callee_pairs.csv'
output_filename = 'data/top100.md'

top_x_limit = 100

# This list is manually updated. Would be nice to have automated.
oeis_number_of_refs = {
    10051 => 1040,
    40 => 9520,
    10 => 3307,
    5 => 3852,
    142 => 2319,
    244049 => 2,
    20639 => 829,
    230980 => 2,
    4086 => 442,
    206735 => 2,
    8683 => 1157,
    301657 => 1,
    293810 => 1,
    105661 => 3,
    7947 => 665,
    117818 => 3,
    3415 => 478,
    52126 => 109,
    203 => 4060,
    65090 => 19,
    25676 => 1,
    191107 => 8,
    86436 => 9,
    3961 => 458,
    1316 => 183,
    138342 => 2,
    33142 => 0,
    80578 => 15,
    134816 => 21,
    166260 => 1,
    195128 => 1,
    73869 => 5,
    62967 => 1,
    60144 => 11,
    14684 => 13,
    282162 => 15,
    6530 => 877,
    189662 => 5,
    236840 => 20,
    32 => 1183,
    7318 => 1854,
    66628 => 3,
    73093 => 17,
    29883 => 12,
    64989 => 300,
    88580 => 22,
    2487 => 344,
    89196 => 1,
    97133 => 3,
    196 => 298,
}

# Obtain all the ranked program_ids
ranked_program_ids = []
CSV.foreach(input_filename0, {:col_sep => ";"}) do |row|
    col0 = row[0]
    program_id = col0.to_i
    next if program_id == 0
    ranked_program_ids << program_id
    break if ranked_program_ids.count == top_x_limit
end

# p ranked_program_ids

program_id_dict = {}
CSV.foreach(input_filename1, {:col_sep => ";"}) do |row|
    col0 = row[0]
    col1 = row[1]
    program_id0 = col0.to_i # caller
    program_id1 = col1.to_i # callee
    next if program_id0 == 0
    next if program_id1 == 0
    
    ary = program_id_dict[program_id1] || []
    ary << program_id0
    program_id_dict[program_id1] = ary
end

# p program_id_dict.first(20)

def oeis_a_name(program_id)
    "A%06i" % program_id
end

def program_link(program_id)
    a_name = oeis_a_name(program_id)
    dir_name = "%03i" % (program_id / 1000)
    "[#{a_name}](https://github.com/ckrause/loda/blob/master/programs/oeis/#{dir_name}/#{a_name}.asm)"
end

def oeis_link(program_id)
    a_name = oeis_a_name(program_id)
    "[#{a_name}](https://oeis.org/#{a_name})"
end

comments = {
    244049 => 'Popular in LODA, underappreciated in OEIS. Why?',
    230980 => 'Popular in LODA, underappreciated in OEIS. Why?',
    206735 => 'Popular in LODA, underappreciated in OEIS. Why?',
}

rows = []
rows << "# Most called LODA programs"
rows << ''
rows << "Rank | LODA (callers) | OEIS (refs) | Comment"
rows << "---- | ---- | ---- | ----"
ranked_program_ids.each_with_index do |program_id, index|
    caller_ary = program_id_dict[program_id] || []
    number_of_callers = caller_ary.count
    
    number_of_oeis_refs = oeis_number_of_refs[program_id] || 'n/a'
    
    comment = comments[program_id] || ''
    
    columns = []
    columns << (index+1).to_s
    columns << program_link(program_id) + " (#{number_of_callers})"
    columns << oeis_link(program_id) + " (#{number_of_oeis_refs})"
    columns << comment
    
    rows << columns.join(' | ')
end
rows << ''

output_content = rows.join("\n")
# puts output_content
IO.write(output_filename, output_content)

puts "Ok, written #{rows.count} lines to file: #{output_filename}"
