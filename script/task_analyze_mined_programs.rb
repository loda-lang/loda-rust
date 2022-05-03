#!/usr/bin/env ruby

=begin
Look up terms in oeis and gather the OEIS ids.

This script traverses all the programs inside the "mine-event" dir.
It looks for all the LODA assembly programs there are.

=end

require_relative 'config'
require 'csv'
require 'set'

class CandidateProgram
    attr_reader :path
    attr_reader :terms40
    attr_reader :oeis_ids
    
    def initialize(path, terms40)
        @path = path
        @terms40 = terms40
        @oeis_ids = []
    end
    
    def append_oeis_id(oeis_id)
        @oeis_ids << oeis_id
    end
end

MINE_EVENT_DIR = Config.instance.dot_loda_rust_mine_event
unless Dir.exist?(MINE_EVENT_DIR)
    raise "No such dir #{MINE_EVENT_DIR}, cannot run script"
end

LODA_CPP_EXECUTABLE = Config.instance.loda_cpp_executable
unless File.exist?(LODA_CPP_EXECUTABLE)
    raise "No such file #{LODA_CPP_EXECUTABLE}, cannot run script"
end

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
unless File.exist?(LODA_PROGRAMS_OEIS)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

OEIS_STRIPPED_FILE = Config.instance.oeis_stripped_file
unless File.exist?(OEIS_STRIPPED_FILE)
    raise "No such file #{OEIS_STRIPPED_FILE}, cannot run script"
end

ANALYTICS_DIR_DONT_MINE_FILE = Config.instance.analytics_dir_dont_mine_file
unless File.exist?(ANALYTICS_DIR_DONT_MINE_FILE)
    raise "No such file #{ANALYTICS_DIR_DONT_MINE_FILE}, cannot run script"
end

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

paths = absolute_paths_for_all_programs(MINE_EVENT_DIR)
if paths.empty?
    puts "There are no pending programs to be processed."
    exit 0
end
puts "Number of programs to be analyzed: #{paths.count}"

dontmine_program_id_set = Set.new
CSV.foreach(ANALYTICS_DIR_DONT_MINE_FILE, col_sep: ";") do |row|
    col0 = row[0]
    program_id = col0.to_i
    next if program_id == 0
    dontmine_program_id_set << program_id
end
puts "Number of items in dontmine file: #{dontmine_program_id_set.count}"

# This script traverses all the programs inside the "mine-event" dir.
# It evaluate all the LODA assembly programs, and obtain 40 terms.
candidate_programs = []
count_success = 0
count_failure = 0
paths.each_with_index do |path, index|
    #puts "#{index} #{path}"
    # puts "---A"
    output = `#{LODA_CPP_EXECUTABLE} eval #{path} -t 40`
    output.strip!
    success = $?.success?
    if success
        count_success += 1
        candidate_programs << CandidateProgram.new(path, output)
    else
        puts "Couldn't eval program at path: #{path}"
        puts output
        # puts "---B"
        count_failure += 1
    end
    
    # break if index > 10
end


puts "evaluate: count_success: #{count_success}  count_failure: #{count_failure}"

#p candidate_programs

# Look up the 40 terms and gather the OEIS ids matches.
approx_row_count = 350000
File.new(OEIS_STRIPPED_FILE, "r").each_with_index do |line, index|
    if (index % 10000) == 0
        percentage = (100 * index) / approx_row_count
        puts "progress %#{percentage}  #{index}/#{approx_row_count}"
    end
    next unless line =~ /^A0*(\d+) ,(.+)$/
    program_id = $1.to_i
    all_terms = $2
    
    # skip this line if the program_id is contained in the "dont_mine.csv", so that duplicate and unwanted sequences never gets submitted.
    next if dontmine_program_id_set.include?(program_id)
    
    candidate_programs.each do |candidate_program|
        if all_terms.start_with?(candidate_program.terms40)
            # puts "#{program_id} #{candidate_program.terms40}"
            candidate_program.append_oeis_id(program_id)
        end
    end
    
    # break if index > 20000
end

#p candidate_programs

def path_for_oeis_program(program_id)
    filename = "A%06i.asm" % program_id
    dirname = "%03i" % (program_id / 1000)
    File.join(LODA_PROGRAMS_OEIS, dirname, filename)
end

def analyze_candidate(candidate_program, program_id)
    path = path_for_oeis_program(program_id)
    if File.exist?(path)
        puts "ignoring #{program_id}, since there already is a program with that id. path: #{path}"
        return false
    end
    puts "Creating file: #{path}"
    IO.write(path, IO.read(candidate_program.path))

    a_name = "A%06i" % program_id

    output = `#{LODA_CPP_EXECUTABLE} check #{a_name} -b 0`
    output.strip!
    success = $?.success?
    if !success
        puts "check failure"
        puts output
        raise "check failure"
    end
    puts "check success"
    puts output
    unless output =~ /^(\d+) .* expected/
        raise "regex didn't match"
    end
    correct_term_count = $1.to_i
    puts "correct #{correct_term_count} terms, followed by mismatch"
    path_deleted = path + "_deleted_different"
    File.rename(path, path_deleted)
    
    # save to mismatch dir
    mismatch_name = "#{a_name}_#{correct_term_count}.asm"
    mismatch_path = File.join(MINE_EVENT_DIR, mismatch_name)
    IO.write(mismatch_path, IO.read(candidate_program.path))
    return true
end

def process_candidate_program(candidate_program)
    raise unless candidate_program.kind_of?(CandidateProgram)
    program_ids = candidate_program.oeis_ids
    puts "Checking: #{candidate_program.path}  candidate program_ids: #{program_ids}"
    reject_candidate = true
    program_ids.each do |program_id|
        if analyze_candidate(candidate_program, program_id)
            reject_candidate = false
        end
    end
    
    if reject_candidate
        puts "Reject candidate program. It doesn't match with all the terms or it's too slow"
        return
    end

    # delete candidate program when it has been fully analyzed
    path_deleted = candidate_program.path + "_deleted_candidate"
    File.rename(candidate_program.path, path_deleted)
    puts "Successfully mined a program"
end

def process_candidate_programs(candidate_programs)
    if candidate_programs.empty?
        raise "no candidate programs to process"
    end
    #candidate_programs = [candidate_programs.first]
    candidate_programs.each do |candidate_program|
        process_candidate_program(candidate_program)
    end
end

process_candidate_programs(candidate_programs)
