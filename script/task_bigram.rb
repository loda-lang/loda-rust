#!/usr/bin/env ruby

=begin
Creates a bigram with LODA instructions.
https://en.wikipedia.org/wiki/N-gram

Prerequisits:
The 'dotenv' gem. Install it using `$ gem install dotenv`
https://github.com/bkeepers/dotenv

This script traverses all the programs inside the LODA program rootdir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent combinations of instructions.

This script outputs a `bigram.csv` file, with this format:

    count;word0;word1
    18066;mov;mov
    14712;mov;lpb
    13387;mov;sub
    13132;mov;add
    11776;add;mov
    10522;add;add

Learnings from this bigram with LODA programs:
Learning A: The `mov` instruction is most likely to be followed by another `mov` instruction.
Learning B: The `mul` instruction is most likely to be followed by an `add` instruction.
Learning C: The `lpb` instruction is most likely to be followed by a `mov` instruction.
=end

require 'csv'
require 'dotenv'
Dotenv.load('../.env')

LODA_PROGRAM_ROOTDIR = ENV['LODA_PROGRAM_ROOTDIR']

output_filename = 'data/bigram.csv'

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def bigrams_from_file(path)
    content = IO.read(path)
    matches = content.scan(/^\s*(\w{2,4})\b/)
    words = matches.flatten
    combos = []
    words.each_cons(2) do |word0, word1|
        combos << "#{word0};#{word1}"
    end
    combos
end

def process_files(paths)
    dict = {}
    paths.map do |path|
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        combos = bigrams_from_file(path)
        combos.each do |combo|
            dict[combo] = (dict[combo] || 0) + 1
        end
    end
    dict
end

paths = absolute_paths_for_all_programs(LODA_PROGRAM_ROOTDIR)
# paths = paths.first(10)
count_combo_dict = process_files(paths)
#puts "count: #{count_combo_dict.count}"

# Convert from dictionary to array
count_combo_ary = count_combo_dict.to_a.map {|combo,count| [count, combo] }

# Move the most frequently occuring items to the top
# Move the lesser used items to the bottom
count_combo_ary = count_combo_ary.sort.reverse

CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["count", "word0", "word1"]
    count_combo_ary.each_with_index do |count_combo, index|
        count, combo = count_combo
        words = combo.split(';')
        row = [count] + words
        csv << row
        # break if index == 10
    end
end
