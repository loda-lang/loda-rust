#!/usr/bin/env ruby

=begin
Creates a skipgram with LODA instructions.
https://en.wikipedia.org/wiki/N-gram

This script traverses all the programs inside the LODA program rootdir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent combinations of instructions.

This script outputs a `skipgram.csv` file, with this format:

    count;word0;word1;word2
    17826;mov;skip;add
    15585;mov;skip;mov
    12458;mov;skip;lpb
    11971;add;skip;mov
    11942;mov;skip;sub
    11662;sub;skip;mov

Learnings from this skipgram with LODA programs:
Learning A: The `mov` and some junk is usually followed by the `add` instruction.
Learning B: The `add` and some junk is usually followed by the `mov` instruction.
Learning C: The `sub` and some junk is usually followed by the `mov` instruction.
=end

require 'csv'
require_relative 'config'

LODA_PROGRAM_ROOTDIR = Config.instance.loda_program_rootdir

output_filename = 'data/skipgram.csv'

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def skipgrams_from_file(path)
    content = IO.read(path)
    matches = content.scan(/^\s*(\w{2,4})\b/)
    words = ['START'] + matches.flatten + ['STOP']
    combos = []
    words.each_cons(3) do |word0, word1, word2|
        combos << "#{word0};skip;#{word2}"
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
        combos = skipgrams_from_file(path)
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
    csv << ["count", "word0", "word1", "word2"]
    count_combo_ary.each_with_index do |count_combo, index|
        count, combo = count_combo
        words = combo.split(';')
        row = [count] + words
        csv << row
        # break if index == 10
    end
end
