#!/usr/bin/env ruby

=begin
Look up terms in oeis and gather the OEIS ids.

This script traverses all the programs inside the "mine-event" dir.
It looks for all the LODA assembly programs there are.

=end

require_relative 'config'
require 'csv'
require 'set'
require 'date'

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
approx_row_count = 400000
File.new(OEIS_STRIPPED_FILE, "r").each_with_index do |line, index|
    if (index % 50000) == 0
        percentage = (100 * index) / approx_row_count
        puts "progress %#{percentage}  #{index}/#{approx_row_count}"
    end
    next unless line =~ /^A0*(\d+) ,(.+)$/
    program_id = $1.to_i
    all_terms = $2
    
    candidate_programs.each do |candidate_program|
        if all_terms.start_with?(candidate_program.terms40)
            # puts "#{program_id} #{candidate_program.terms40}"
            candidate_program.append_oeis_id(program_id)
        end
    end
    
    # break if index > 20000
end

#p candidate_programs

def loda_eval_steps(path_program)
    command_output = `#{LODA_CPP_EXECUTABLE} eval #{path_program} -t 60 -s`
    command_output.strip!
    unless $?.success?
        raise "loda eval step"
    end
    command_output.split(',').map { |s| s.to_i }
end

def compare_performance_lodasteps(path_program0, path_program1, path_benchmark)
    steps0 = loda_eval_steps(path_program0)
    steps1 = loda_eval_steps(path_program1)
    sum0 = steps0.sum
    sum1 = steps1.sum
    step0_less_than_step1 = 0
    last_slice_step0_greater_than_step1 = 0
    step0_same_step1 = 0
    step0_greater_than_step1 = 0
    identical = true
    benchmark_rows = []
    steps0.zip(steps1).each_with_index do |pair, index|
        step0, step1 = pair
        comparison_symbol = " "
        if step0 == step1
            step0_same_step1 += 1
            comparison_symbol = " = "
        end
        if step0 > step1
            step0_greater_than_step1 += 1
            comparison_symbol = "  >"
            identical = false
            if index > 10
                last_slice_step0_greater_than_step1 += 1
            end
        end
        if step0 < step1
            step0_less_than_step1 += 1
            comparison_symbol = "<  "
            identical = false
        end
        benchmark_rows << ("%10i %s %10i" % [step0, comparison_symbol, step1])
    end
    result = :undecided
    reason = :undecided
    while true
        if identical
            result = :program1
            reason = "identical number of steps as the existing program"
            break
        end
        if sum0 == sum1
            result = :program1
            reason = "same sum as the existing program"
            break
        end
        if sum0 > sum1
            result = :program1
            reason = "total sum of new program is greater than existing program"
            break
        end
        if last_slice_step0_greater_than_step1 > 0
            result = :program1
            reason = "total sum of new program is greater than existing program"
            break
        end
        if sum0 < sum1
            result = :program0
            reason = "the new program is faster than the existing program"
            break
        end
        result = :program1
        reason = "uncaught scenario. Using existing program"
        break
    end
    benchmark_summary_rows = []
    benchmark_summary_rows << "Result: #{result}"
    benchmark_summary_rows << "Reason: #{reason}"
    benchmark_summary_rows << ""
    benchmark_summary_rows << ("SUM: %10i %s %10i" % [sum0, " ", sum1])
    rows = benchmark_summary_rows + benchmark_rows
    content = rows.join("\n") + "\n"
    IO.write(path_benchmark, content)
    result
end

def path_for_oeis_program(program_id)
    filename = "A%06i.asm" % program_id
    dirname = "%03i" % (program_id / 1000)
    File.join(LODA_PROGRAMS_OEIS, dirname, filename)
end

def analyze_candidate(candidate_program, program_id)
    path = path_for_oeis_program(program_id)
    milliseconds = DateTime.now.strftime('%Q')
    path_original = path + "_original_#{milliseconds}"
    path_reject = path + "_reject_#{milliseconds}"
    path_check_output = path + "_check_output_#{milliseconds}"
    path_benchmark = path + "_benchmark_#{milliseconds}"
    has_original_file = File.exist?(path)
    if has_original_file
        # puts "There already exist program: #{program_id}, Renaming from: #{path} to: #{path_original}"
        File.rename(path, path_original)
    else
        # puts "No existing program exist for: #{program_id}"
    end
    
    loda_minimize_output = `#{LODA_CPP_EXECUTABLE} minimize #{candidate_program.path}`
    loda_minimize_output = loda_minimize_output.strip + "\n"
    unless $?.success?
        puts "loda minimize, expected exit code 0, but got exit code: #{$?}, see loda check output: #{path_check_output}"
        puts loda_minimize_output
        raise "loda minimize exit code"
    end
    
    # puts "Creating file: #{path}"
    IO.write(path, loda_minimize_output)

    a_name = "A%06i" % program_id

    loda_check_output = `#{LODA_CPP_EXECUTABLE} check #{a_name} -b 0 > #{path_check_output}`
    loda_check_output.strip!
    unless $?.success?
        puts "loda check, expected exit code 0, but got exit code: #{$?}, see loda check output: #{path_check_output}"
        puts loda_check_output
        raise "loda check exit code"
    end
    check_output_content = IO.read(path_check_output)
    if check_output_content =~ /^std::exception$/
        puts "Rejecting. c++ exception occurred, probably due to overflow or cyclic dependency. see output: #{path_check_output}."
        File.rename(path, path_reject)
        if has_original_file
            File.rename(path_original, path)
        end
        return false
    end
    if check_output_content =~ /^ok$/
        if !has_original_file
            puts "Keeping. This program is new, there is no previous implementation."
            return true
        end
        
        # Compare performance new program vs old program
        comparision_id = compare_performance_lodasteps(path, path_original, path_benchmark)

        # If the new program is faster, then keep it, otherwise reject it.
        if comparision_id == :program1
            puts "Rejecting. This program isn't better than the existing program."
            File.rename(path, path_reject)
            if has_original_file
                File.rename(path_original, path)
            end
            return false
        end
        if comparision_id == :program0
            puts "Keeping. This program is faster than the old implementation."
            return true
        end
        raise "unknown comparison result #{comparison_id}"
    end
    unless check_output_content =~ /^(\d+) .* expected/
        raise "Regex didn't match. See bottom of the file: #{path_check_output} Perhaps 'loda check' have changed its output format."
    end
    correct_term_count = $1.to_i
    puts "Keeping. This program is a mismatch, it has correct #{correct_term_count} terms, followed by mismatch"
    path_deleted = path + "_deleted_different"
    File.rename(path, path_deleted)
    if has_original_file
        File.rename(path_original, path)
    end
    
    # save to mismatch dir
    mismatch_name = "#{a_name}_#{correct_term_count}_0.asm"
    mismatch_path = File.join(MINE_EVENT_DIR, mismatch_name)
    IO.write(mismatch_path, IO.read(candidate_program.path))
    return true
end

def process_candidate_program(candidate_program, dontmine_program_id_set)
    raise unless candidate_program.kind_of?(CandidateProgram)
    program_ids = candidate_program.oeis_ids
    if program_ids.empty?
        puts "Ignoring candidate program. There isn't any candidate program ids for '#{candidate_program.path}', this happes when there are less than 40 known terms in the stripped.zip file"
        return
    end
    puts "\n\nChecking: #{candidate_program.path}  candidate program_ids: #{program_ids}"
    reject_candidate = true
    program_ids.each do |program_id|
        # The "dont_mine.csv" holds program_ids of unwanted sequences, duplicates, protected programs and stuff that is not to be mined.
        if dontmine_program_id_set.include?(program_id)
            puts "Skip candidate program id, which is contained in the 'dont_mine.csv' file. A#{program_id}"
            next
        end
        if analyze_candidate(candidate_program, program_id)
            puts "This is a keeper a keeper. A#{program_id}"
            reject_candidate = false
        end
    end
    if reject_candidate
        # Rename program when it has been fully analyzed
        path_reject = candidate_program.path + "_status_reject"
        File.rename(candidate_program.path, path_reject)
        puts "Status: Rejecting bad program. #{path_reject}"
        return
    end

    # Rename program when it has been fully analyzed
    path_keep = candidate_program.path + "_status_keep"
    File.rename(candidate_program.path, path_keep)
    puts "Status: Keeping good program. #{path_keep}"
end

def process_candidate_programs(candidate_programs, dontmine_program_id_set)
    if candidate_programs.empty?
        raise "no candidate programs to process"
    end
    #candidate_programs = [candidate_programs.first]
    candidate_programs.each do |candidate_program|
        process_candidate_program(candidate_program, dontmine_program_id_set)
    end
end

process_candidate_programs(candidate_programs, dontmine_program_id_set)
