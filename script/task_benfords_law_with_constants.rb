#!/usr/bin/env ruby

=begin

Determines the distribution of the first digit.

This script analyze the constants used across all LODA programs.
Ignoring oeis program_ids.
Ignoring the digit 0.
Ignoring comments.
Ignoring target registers.
Ignoring source registers.
The minus sign is ignored if it's a negative constant.
Only the first digit is considered. The remaning digits are ignored.

Benford's Law
https://www.youtube.com/watch?v=vIsDjbhbADY

Output from this program looks like this:
digit;count
1;122724
2;71614
3;21240
4;16455
5;9411
6;8057
7;4741
8;5170
9;4096

=end

require_relative 'config'
require 'csv'

CACHE_DIR = Config.instance.cache_dir
unless Dir.exist?(CACHE_DIR)
    raise "No such dir #{CACHE_DIR}, cannot run script"
end

CACHE_HISTOGRAM_INSTRUCTION_CONSTANT_CSV = File.join(CACHE_DIR, 'histogram_instruction_constant.csv')

dict = Hash.new(0)
CSV.foreach(CACHE_HISTOGRAM_INSTRUCTION_CONSTANT_CSV, col_sep: ";") do |row|
    count = row[0].to_i
    value = row[2].to_i
    next if count == 0
    next if value == 0
    value_string = value.abs.to_s
    first_digit_string = value_string[0]
    first_digit = first_digit_string.to_i
    dict[first_digit] += count
end

puts "Distribution of first digits"
puts
puts "digit;count"
(1..9).each do |i|
    count = dict[i]
    puts "#{i};#{count}"
end
