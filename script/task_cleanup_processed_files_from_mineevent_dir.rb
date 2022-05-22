#!/usr/bin/env ruby

=begin
This script removes ".asm_status_reject" and ".asm_status_keep" files.

After analyzing the mined programs, the "mine-event" dir is left with files like these:
20220522-182722-410927334.asm_status_reject
20220522-183346-423905535.asm_status_keep
=end

require_relative 'config'

MINE_EVENT_DIR = Config.instance.dot_loda_rust_mine_event
unless Dir.exist?(MINE_EVENT_DIR)
    raise "No such dir #{MINE_EVENT_DIR}, cannot run script"
end

def absolute_paths_for_all_processed_programs(rootdir)
    relative_paths0 = Dir.glob(File.join("**", "*.asm_status_reject"), base: rootdir)
    relative_paths1 = Dir.glob(File.join("**", "*.asm_status_keep"), base: rootdir)
    relative_paths = (relative_paths0 + relative_paths1).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

# Identify all the files that are to be deleted
files_to_be_deleted = absolute_paths_for_all_processed_programs(MINE_EVENT_DIR)
puts "Number of files to be deleted from mine-event dir: #{files_to_be_deleted.count}"

files_to_be_deleted.each do |path|
    File.delete(path)
end
