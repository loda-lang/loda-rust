#!/usr/bin/env ruby

=begin
This script traverses all the programs inside the "loda-programs/oeis" dir.
It looks for all the LODA assembly programs there are.
This script determines the most frequent used constants that goes with each instruction.

This script outputs a `constants.csv` file, with this format:

    count;instruction;constant
    532;add;1
    531;sub;1
    308;mov;1
    252;mul;2
    167;div;2
    137;mov;2
    121;add;2
    98;pow;2
    78;cmp;0
    69;bin;2

=end

require 'csv'
require 'date'
require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis

output_filename = 'data/constants.csv'

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def populate_dictionary_with_file_stats(dict, path)
    content = IO.read(path)
    matches = content.scan(/^\s*(\w{2,4})\s+[$]{1,2}\d+\s*,\s*(-?\d+)\b/)
    matches.each do |instruction, constant|
        if instruction == 'seq'
            next
        end
        if instruction == 'lpb'
            next
        end
        key = "#{instruction} #{constant}"
        dict[key] = (dict[key] || 0) + 1
    end
 end

def process_files(paths)
    dict = {}
    time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    paths_count = paths.count
    progress_n = [(paths_count / 30), 1].max
    number_of_rows = 0
    paths.each_with_index do |path, index|
        if (index % progress_n) == 0
            percent = (100 * index).to_f / paths_count
            percent_s = "%.2f" % percent
            time_end = Process.clock_gettime(Process::CLOCK_MONOTONIC)
            time_elapsed = time_end - time_start
            time_elapsed_s = "%.3f" % time_elapsed
            puts "progress: #{index}/#{paths_count}, %#{percent_s}  rows: #{number_of_rows}  elapsed: #{time_elapsed_s}"
        end
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        populate_dictionary_with_file_stats(dict, path)
        number_of_rows = dict.count
    end
    dict
end

paths = absolute_paths_for_all_programs(LODA_PROGRAMS_OEIS)
#paths = paths.first(10)
# paths = paths.first(1000)
instruction_constant_count_dict = process_files(paths)
#puts "count: #{instruction_constant_count_dict.count}"

# Convert from dictionary to array
count_combo_ary = instruction_constant_count_dict.to_a.map {|combo,count| [count, combo] }

# Move the most frequently occuring items to the top
# Move the lesser used items to the bottom
count_combo_ary = count_combo_ary.sort.reverse

CSV.open(output_filename, "wb", col_sep: ";") do |csv|
    csv << ["count", "instruction", "constant"]
    count_combo_ary.each_with_index do |count_combo, index|
        count, combo = count_combo
        words = combo.split(' ')
        row = [count] + words
        csv << row
        # break if index == 10
    end
end
