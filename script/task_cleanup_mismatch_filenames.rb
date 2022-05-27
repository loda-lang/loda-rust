#!/usr/bin/env ruby

=begin
The naming of mismatches often ends up with inconsistent naming.
This script cleans up the indexes, so they start from 0 and increments for each mismatch variant.
=end

require 'csv'
require_relative 'config'

class FilenameWithRank
    attr_reader :path
    attr_reader :filename_part1
    attr_reader :primary_index
    
    def initialize(path, filename_part1, primary_index)
        @path = path
        @filename_part1 = filename_part1
        @primary_index = primary_index
    end
    
    def update_primary_index(new_primary_index)
        @primary_index = new_primary_index
    end
    
    def source_filename
        File.basename(@path)
    end
    
    def destination_filename
        "#{@filename_part1}_#{@primary_index}.asm"
    end
    
    def destination_path
        File.join(File.dirname(@path), destination_filename)
    end
end

LODA_RUST_MISMATCHES = Config.instance.loda_outlier_programs_repository_oeis_divergent
unless Dir.exist?(LODA_RUST_MISMATCHES)
    raise "No such dir #{LODA_RUST_MISMATCHES}, cannot run script"
end

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end


# Identify all the files that are to be renamed
paths = absolute_paths_for_all_programs(LODA_RUST_MISMATCHES)
puts "Total number of files: #{paths.count}" 
filename_with_rank_ary = []
paths.each do |path|
    extension = File.extname(path)
    next unless extension == '.asm'
    filename = File.basename(path, '.asm')
    
    re = /^(A\d+_\d+)_?(.*)$/
    if filename =~ re
        filename_part1 = $1
        filename_part2 = $2
        primary_index = -1
        if filename_part2 =~ /^\d+$/
            primary_index = filename_part2.to_i
        end
        # p primary_index
        filename_with_rank = FilenameWithRank.new(path, filename_part1, primary_index)
        filename_with_rank_ary << filename_with_rank
    end
end

# Group related files together in the same array
# Use the filename_part1 as the key, eg. "A123456_800"
groups = {}
filename_with_rank_ary.each do |filename_with_rank|
    groups[filename_with_rank.filename_part1] = (groups[filename_with_rank.filename_part1] || []) + [filename_with_rank]
end

# Sort related files by their index
# If the file has no meaningful index, then assign an unique index
# Preserve as much as possible the originally assigned index, so that the git-diff is as small as possible.
groups.each do |key, items|
    before = items.map {|item| item.primary_index }
    items = items.sort_by { |a| a.primary_index }
    
    ary_front = []
    ary_back = []
    items.each do |item|
        if item.primary_index >= 0
            ary_front << item
        else
            ary_back << item
        end
    end
    ary = ary_front + ary_back
    
    ary.each_with_index do |item, index|
        item.update_primary_index(index)
    end

    after = items.map {|item| item.primary_index }
    groups[key] = ary
    # if before != after
    #     puts "sorted: #{before} -> #{after}"
    # end
end

# Determine the order the files are to be renamed
filename_with_rank_ary_ordered = []
keys_sorted = groups.keys.sort
keys_sorted.each do |key|
    items = groups[key]
    items.each do |item|
        pretty_source = "%-20s" % item.source_filename
        pretty_destination = "%-20s" % item.destination_filename
        # puts "#{pretty_source} ; #{pretty_destination}"
        filename_with_rank_ary_ordered << item
    end
end

# Perform the renaming
number_of_renamed_files = 0
filename_with_rank_ary_ordered.each do |filename_with_rank|
    source = filename_with_rank.path
    dest = filename_with_rank.destination_path
    next if source == dest
    puts "rename\n  from: #{source}\n  to:   #{dest}"
    number_of_renamed_files += 1
    File.rename(source, dest)
end

puts "Number of renamed files: #{number_of_renamed_files}"
