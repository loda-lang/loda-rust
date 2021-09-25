#!/usr/bin/env ruby

=begin
Creates a trigram with LODA instructions.
https://en.wikipedia.org/wiki/N-gram

This script traverses all the programs inside the "loda-programs/oeis" dir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent combinations of instructions.

This script outputs a `trigram.csv` file, with this format:

    count;word0;word1;word2
    8776;mov;mov;lpb
    6709;lpb;mov;sub
    5717;START;mov;mov
    5386;mov;lpb;mov
    4321;mov;lpb;sub
    4310;mul;add;STOP

Learnings from this trigram with LODA programs:
Learning A: The `mov` and `mov` is usually followed by a `lpb` instruction.
Learning B: The `lpb` and `mov` is usually followed by a `sub` instruction.
Learning C: The `mov` and `lpb` is usually followed by a `mov` instruction.
Learning D: The `mul` and `add` is usually the last of the program.
=end

require 'csv'
require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis

output_filename = 'data/trigram.csv'

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def trigrams_from_file(path)
    content = IO.read(path)
    matches = content.scan(/^\s*(\w{2,4})\b/)
    words = ['START'] + matches.flatten + ['STOP']
    combos = []
    words.each_cons(3) do |word0, word1, word2|
        combos << "#{word0};#{word1};#{word2}"
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
        combos = trigrams_from_file(path)
        combos.each do |combo|
            dict[combo] = (dict[combo] || 0) + 1
        end
    end
    dict
end

paths = absolute_paths_for_all_programs(LODA_PROGRAMS_OEIS)
# paths = paths.first(10)
count_combo_dict = process_files(paths)
#puts "count: #{count_combo_dict.count}"

# Convert from dictionary to array
count_combo_ary = count_combo_dict.to_a.map {|combo,count| [count, combo] }

# Move the most frequently occuring items to the top
# Move the lesser used items to the bottom
count_combo_ary = count_combo_ary.sort.reverse

CSV.open(output_filename, "wb", col_sep: ";") do |csv|
    csv << ["count", "word0", "word1", "word2"]
    count_combo_ary.each_with_index do |count_combo, index|
        count, combo = count_combo
        words = combo.split(';')
        row = [count] + words
        csv << row
        # break if index == 10
    end
end
