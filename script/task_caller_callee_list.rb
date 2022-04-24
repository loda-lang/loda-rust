#!/usr/bin/env ruby

=begin

This script takes input from a `analytics/dependencies.csv` file, with this format:

    caller program id;callee program id
    73;232508
    73;301657
    134;22844
    134;121381
    134;246388
    134;4082

This script outputs a `caller_callee_list.csv` file, with this format:

    program id;dependency count;program ids
    4;1;4
    5;1;5
    7;1;7
    8;2;8,165190
    10;1;10

=end

require 'csv'
require 'set'
require_relative 'config'

INPUT_FILENAME = Config.instance.analytics_dir_dependencies_file
OUTPUT_FILENAME = 'data/caller_callee_list.csv'

class Graph
    def initialize(direct_dependencies)
        @direct_dependencies = direct_dependencies
    end
    
    def indirect_dependencies(program_id)
        result = Set.new
        indirect_dependencies_inner(program_id, result)
        result.delete(program_id)
        return result
    end
    
    def indirect_dependencies_inner(program_id, result)
        if result.member?(program_id)
            return
        end
        result.add(program_id)
        set = @direct_dependencies[program_id]
        if set == nil
            return
        end
        set.each do |nested_program_id|
            indirect_dependencies_inner(nested_program_id, result)
        end
    end
end

time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)

# Obtain all the program_ids to be processed
dependency_edges = []
CSV.foreach(INPUT_FILENAME, col_sep: ";") do |columns|
    col0 = columns[0]
    col1 = columns[1]
    program_id0 = col0.to_i
    program_id1 = col1.to_i
    next if program_id0 == 0
    next if program_id1 == 0
    dependency_edges << [program_id0, program_id1]
end

# Resolve direct dependencies
direct_dependencies_dict = {}
dependency_edges.each do |caller_program_id, callee_program_id|
    set = direct_dependencies_dict[caller_program_id] || Set.new
    set.add(callee_program_id)
    direct_dependencies_dict[caller_program_id] = set
end
program_ids_count = direct_dependencies_dict.count

# Resolve indirect dependencies 
graph = Graph.new(direct_dependencies_dict)
dict = {}
direct_dependencies_dict.keys.each do |program_id|
    dict[program_id] = graph.indirect_dependencies(program_id)
end

# Generate output file
CSV.open(OUTPUT_FILENAME, "wb", col_sep: ";") do |csv|
    csv << ["program id", "dependency count", "program ids"]
    dict.each do |program_id, callee_program_id_set|
        callee_program_ids = callee_program_id_set.to_a.sort
        callee_program_ids_string = callee_program_ids.join(",")
        dependency_count = callee_program_ids.count
        csv << [program_id.to_s, dependency_count.to_s, callee_program_ids_string]
    end
end

# Show stats
time_end = Process.clock_gettime(Process::CLOCK_MONOTONIC)
time_elapsed = time_end - time_start
time_elapsed_s = "%.3f" % time_elapsed
puts "elapsed: #{time_elapsed_s}"
