#!/usr/bin/env ruby

=begin
=end

require 'csv'
require_relative 'config'

class FilenameWithRank
    attr_reader :path
    
    def initialize(path)
        @path
    end
end

LODA_RUST_MISMATCHES = Config.instance.loda_rust_mismatches
unless Dir.exist?(LODA_RUST_MISMATCHES)
    raise "No such dir #{LODA_RUST_MISMATCHES}, cannot run script"
end

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

paths = absolute_paths_for_all_programs(LODA_RUST_MISMATCHES)
p paths.count

paths.each do |path|
    extension = File.extname(path)
    next unless extension = '.asm'
    filename = File.basename(path, '.asm')
    
    
    re = /^(A\d+_\d+)(.*)$/
    if filename =~ re
        p $2
    end
    # parts = filename.split('_')
    # sequence_id = parts[0]
    # number_of_terms = parts[1]
    # index = parts[2]
    # p sequence_id
end

