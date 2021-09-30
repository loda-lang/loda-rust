#!/usr/bin/env ruby

=begin
Look up terms in oeis and gather the OEIS ids.

This script traverses all the programs inside the "mine-event" dir.
It looks for all the LODA assembly programs there are.

=end

require_relative 'config'

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

OEIS_STRIPPED_FILE = Config.instance.oeis_stripped_file
unless File.exist?(OEIS_STRIPPED_FILE)
    raise "No such file #{OEIS_STRIPPED_FILE}, cannot run script"
end

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

paths = absolute_paths_for_all_programs(MINE_EVENT_DIR)
puts "Number of programs to be analyzed: #{paths.count}"

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
    candidate_programs.each do |candidate_program|
        if all_terms.start_with?(candidate_program.terms40)
            # puts "#{program_id} #{candidate_program.terms40}"
            candidate_program.append_oeis_id(program_id)
        end
    end
    
    # break if index > 20000
end

p candidate_programs

candidate_programs.each do |candidate_program|
    oeis_ids = candidate_program.oeis_ids
    puts "Checking: #{candidate_program.path}  candidate oeis_ids: #{oeis_ids}"
    oeis_ids.each do |oeis_id|
        
    end
end


