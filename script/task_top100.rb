#!/usr/bin/env ruby

=begin

This script takes input from `analytics/program_rank.csv` file, with this format:

    program id;score
    10051;533.2628
    40;146.3573
    203;44.7505
    10;39.7436
    5171;38.9892
    6005;34.4878

This script takes input from a `analytics/dependencies.csv` file, with this format:

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
require 'set'
require_relative 'config'

OEIS_NAMES_FILE = Config.instance.oeis_names_file
INPUT_FILENAME0 = Config.instance.analytics_dir_dependencies_file
INPUT_FILENAME1 = Config.instance.analytics_dir_program_rank_file
OUTPUT_FILENAME = 'data/top100.md'

top_x_limit = 100

# This list is manually updated. Would be nice to have automated.
oeis_number_of_refs = {
    10 => 3307,
    10051 => 1040,
    10055 => 76,
    10059 => 48,
    10060 => 527,
    101776 => 5,
    10201 => 0,
    105661 => 3,
    1065 => 433,
    106729 => 9,
    1075 => 93,
    108 => 3500,
    114986 => 17,
    1158 => 129,
    117818 => 3,
    120 => 1479,
    1221 => 1606,
    1222 => 2010,
    1223 => 598,
    1227 => 355,
    128634 => 2,
    129 => 647,
    130568 => 3,
    130665 => 26,
    131326 => 5,
    1316 => 183,
    132106 => 17,
    134816 => 21,
    134860 => 14,
    137319 => 11,
    138288 => 11,
    138342 => 2,
    142 => 2319,
    146076 => 26,
    14684 => 13,
    14739 => 10,
    151800 => 155,
    152966 => 1,
    155085 => 8,
    156595 => 6,
    1588 => 4,
    159477 => 2,
    161411 => 12,
    161560 => 1,
    163271 => 9,
    166260 => 1,
    167821 => 5,
    168835 => 0,
    171621 => 6,
    172407 => 1,
    173833 => 5,
    175851 => 13,
    180122 => 3,
    182190 => 7,
    184517 => 2,
    187107 => 1,
    189661 => 11,
    189662 => 5,
    191107 => 8,
    194 => 53,
    194029 => 38,
    194920 => 6,
    1951 => 106,
    195128 => 1,
    1953 => 5,
    196 => 298,
    198081 => 1,
    198082 => 1,
    198083 => 1,
    2024 => 229,
    203 => 4060,
    204 => 305,
    20639 => 829,
    206735 => 2,
    211 => 27,
    2110 => 1411,
    212012 => 7,
    22087 => 13,
    228071 => 2,
    230980 => 2,
    2315 => 111,
    232089 => 1,
    232508 => 1,
    23537 => 11,
    236313 => 1,
    23645 => 18,
    236840 => 20,
    239050 => 29,
    240400 => 4,
    244049 => 2,
    2487 => 344,
    2541 => 31,
    25669 => 1,
    25675 => 1,
    25676 => 1,
    25682 => 1,
    25767 => 4,
    25794 => 0,
    271342 => 3,
    276868 => 4,
    276886 => 4,
    279521 => 2,
    282162 => 15,
    28233 => 26,
    284625 => 4,
    284817 => 3,
    285076 => 6,
    286751 => 3,
    286909 => 4,
    2878 => 108,
    288713 => 3,
    293810 => 1,
    29883 => 12,
    30101 => 148,
    301653 => 0,
    301657 => 1,
    307136 => 6,
    31138 => 10,
    32 => 1183,
    320226 => 5,
    32741 => 172,
    32742 => 212,
    33132 => 0,
    33142 => 0,
    33270 => 9,
    336551 => 5,
    337313 => 3,
    338363 => 1,
    33880 => 89,
    33940 => 7,
    339765 => 1,
    3415 => 478,
    3451 => 4,
    3499 => 37,
    3849 => 193,
    38548 => 144,
    38573 => 35,
    3958 => 61,
    3951 => 59,
    3952 => 57,
    3953 => 59,
    3954 => 57,
    3961 => 458,
    3983 => 35,
    3991 => 88,
    40 => 9520,
    40329 => 2,
    4086 => 442,
    4247 => 24,
    45 => 5066,
    46666 => 8,
    4736 => 318,
    4737 => 28,
    48724 => 60,
    48766 => 41,
    48881 => 31,
    48883 => 53,
    49472 => 20,
    4956 => 11,
    49643 => 6,
    49711 => 29,
    5 => 3852,
    51596 => 5,
    5171 => 65,
    52126 => 109,
    52410 => 86,
    5251 => 168,
    52910 => 2,
    52937 => 3,
    5811 => 126,
    5836 => 200,
    6005 => 46,
    60143 => 10,
    60144 => 11,
    60145 => 1,
    60973 => 4,
    62298 => 47,
    62558 => 1,
    62967 => 1,
    63918 => 1,
    64989 => 300,
    65090 => 19,
    6530 => 877,
    66096 => 7,
    66628 => 3,
    69513 => 21,
    7089 => 219,
    70939 => 499,
    71 => 259,
    720 => 1468,
    72668 => 14,
    73093 => 17,
    7318 => 1854,
    73869 => 5,
    74828 => 2,
    75423 => 3,
    77444 => 15,
    77445 => 7,
    77868 => 9,
    77985 => 13,
    78057 => 65,
    7814 => 699,
    78642 => 8,
    7913 => 242,
    7947 => 665,
    7953 => 956,
    80339 => 20,
    80545 => 6,
    80578 => 15,
    80590 => 2,
    80754 => 6,
    80791 => 64,
    81603 => 34,
    82524 => 0,
    82532 => 4,
    82841 => 11,
    8472 => 335,
    8507 => 2,
    86436 => 9,
    8683 => 1157,
    87057 => 8,
    87799 => 6,
    8833 => 82,
    88580 => 22,
    89068 => 9,
    89196 => 1,
    8937 => 35,
    90368 => 20,
    94820 => 13,
    96270 => 35,
    97133 => 3,
    98090 => 33,
    982 => 90,
    98578 => 8,
    99802 => 6,
}

# Obtain all the ranked program_ids
ranked_program_ids = []
CSV.foreach(INPUT_FILENAME0, col_sep: ";") do |row|
    col0 = row[0]
    program_id = col0.to_i
    next if program_id == 0
    ranked_program_ids << program_id
    break if ranked_program_ids.count == top_x_limit
end

# p ranked_program_ids

program_id_dict = {}
CSV.foreach(INPUT_FILENAME1, col_sep: ";") do |row|
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
    "[#{a_name}](https://github.com/loda-lang/loda-programs/tree/main/oeis/#{dir_name}/#{a_name}.asm)"
end

def oeis_link(program_id)
    a_name = oeis_a_name(program_id)
    "[#{a_name}](https://oeis.org/#{a_name})"
end

# Lookup the A number and obtain the corresponding OEIS name
#program_ids_set = [45, 1113].to_set
program_ids_set = ranked_program_ids.to_set
program_id_to_name = {}
File.new(OEIS_NAMES_FILE, "r").each do |line|
    next unless line =~ /^A0*(\d+) (.+)$/
    program_id = $1.to_i
    next unless program_ids_set.include?(program_id)
    oeis_name = $2
    program_id_to_name[program_id] = oeis_name
end
#puts "program_id_to_name.count: #{program_id_to_name.count}"

def format_oeis_name_as_markdown(name)
    name.gsub(/[^ a-zA-Z0-9+,.]/) { |s| '\\' + s }
end

rows = []
rows << "# Most called LODA programs"
rows << ''
rows << "Rank | LODA (callers) | OEIS (refs) | Name"
rows << "---- | ---- | ---- | ----"
ranked_program_ids.each_with_index do |program_id, index|
    caller_ary = program_id_dict[program_id] || []
    number_of_callers = caller_ary.count
    
    number_of_oeis_refs = oeis_number_of_refs[program_id] || 'n/a'
    name = program_id_to_name[program_id] || 'n/a'
    
    columns = []
    columns << (index+1).to_s
    columns << program_link(program_id) + " (#{number_of_callers})"
    columns << oeis_link(program_id) + " (#{number_of_oeis_refs})"
    columns << format_oeis_name_as_markdown(name)
    
    rows << columns.join(' | ')
end
rows << ''

output_content = rows.join("\n")
# puts output_content
IO.write(OUTPUT_FILENAME, output_content)

puts "Ok, written #{rows.count} lines to file: #{OUTPUT_FILENAME}"
