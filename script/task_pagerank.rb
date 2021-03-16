#!/usr/bin/env ruby

=begin

Prerequisit:
This script runs the PageRank algorithm.
https://github.com/louridas/pagerank
Probably: You may have to install this tool yourself.
Get the repository.
Compile what's inside the `cpp` dir, by typing `make`.
https://github.com/louridas/pagerank/tree/master/cpp
This outputs an executable file named `pagerank`.
Move it to "~/bin/pagerank".

Check if you have the `pagerank` executable in the PATH, by running:
PROMPT> which pagerank
/Users/JohnDoe/bin/pagerank

Check that you can execute the `pagerank` executable. Press CTRL+C to stop it from running.
PROMPT> pagerank
alpha = 0.85 convergence = 1e-05 max_iterations = 10000 numeric = 0 delimiter = ' => '
Reading input from stdin...
^C

So far so good. The `pagerank` tool is installed.



This script takes two input files.

Input from `program_ids.csv` file, with this format:

    program id
    4
    5
    7
    8
    10

Input from `caller_callee_pairs.csv` file, with this format:

    caller program id;callee program id
    73;232508
    73;301657
    134;22844
    134;121381
    134;246388
    134;4082


This script applies the PageRank algorithm to the input.

The PageRank tool outputs a temp file `pagerank_raw_output.csv`, with this format.
There's a lot of junk rows for no corresponding program id. Scientific notation is being used. The separator is ' = '.

    0 = 2.87863034720346e-06
    1 = 2.87863034720346e-06
    2 = 2.87863034720346e-06
    3 = 2.87863034720346e-06
    4 = 3.69424213419305e-06
    5 = 0.000474322958227094
    6 = 2.87863034720346e-06

This script cleans up the PageRank data.

This script outputs a `pagerank.csv` file, with this format:

    program id;pagerank
    10051;0.0011710868
    10;0.0004871711
    142;0.0004844773
    5;0.0004742994
    40;0.0004690897
    244049;0.0003187378

=end
require 'csv'
require 'set'

input_filename0 = 'data/program_ids.csv'
input_filename1 = 'data/caller_callee_pairs.csv'
temp_filename = 'data/pagerank_raw_output.csv'
output_filename = 'data/pagerank.csv'

# Obtain a set of all program ids
program_ids = Set.new
CSV.foreach(input_filename0, {:col_sep => ";"}) do |row|
    program_id = row.first.to_i
    next if program_id == 0
    program_ids.add(program_id)
end
puts "number of programs: #{program_ids.count}"


# Make sure that `pagerank` really is installed
which_pagerank_output = `which pagerank`
which_pagerank_output = which_pagerank_output.strip
which_pagerank_success = $?.success?
if (!which_pagerank_success) || (which_pagerank_output !~ /pagerank/)
    raise "No 'pagerank' executable in the PATH. This script depends on it. Abort."
end

# Run the `pagerank` tool
puts "input: #{input_filename1}"
pagerank_output = `pagerank -n -d ';' #{input_filename1} > #{temp_filename}`
puts pagerank_output
pagerank_output = pagerank_output.strip
pagerank_success = $?.success?
if !pagerank_success
    raise "Failure running the 'pagerank' tool"
end

puts "temp_file: #{temp_filename}"
rows = []
count_good = 0
count_skip = 0
CSV.foreach(temp_filename, {:col_sep => " = "}) do |col0, col1|
    program_id = col0.to_i
    if !program_ids.member?(program_id)
        # For some reason the `pagerank` tool outputs filler rows for non-existing entries
        count_skip += 1
        next
    end
    
    pagerank = col1.to_f
    rows << [pagerank, program_id]
    count_good += 1
    # break if count_good > 10
end
puts "count_skip: #{count_skip}"
puts "count_good: #{count_good}"
rows = rows.sort.reverse

puts "output: #{output_filename}"
CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id", "pagerank"]

    rows.each do |pagerank, program_id|
        pretty_pagerank = "%0.10f" % pagerank
        csv << [program_id, pretty_pagerank]
    end
end
