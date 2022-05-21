#!/usr/bin/env ruby

=begin
This script removes ; comments at the bottom of programs.
The programs that gets placed in the mismatches dir often have comments at the bottom, with verbose info about how they were mined.


Example input:
=======
; 3,9,12,19,25,29,35,38,45,51,55,61,67,71,77,80,87,93,97,103,106,113,119,122,129,135,139,145,148,155,161,165,171,177,181,187,190,197,203,207

seq $0,75317
seq $0,90908

; template 111426
; mutation: ReplaceSourceConstantWithoutHistogram
; mutation: ReplaceSourceConstantWithoutHistogram
; mutation: CallMostPopularProgram
; mutation: CallMostPopularProgram
; mutation: CallMostPopularProgram
; keep: performance NewProgramIsAlwaysFaster than 140979
=======

Example output:
=======
; 3,9,12,19,25,29,35,38,45,51,55,61,67,71,77,80,87,93,97,103,106,113,119,122,129,135,139,145,148,155,161,165,171,177,181,187,190,197,203,207

seq $0,75317
seq $0,90908
=======

=end

require_relative 'config'

LODA_RUST_MISMATCHES = Config.instance.loda_rust_mismatches
unless Dir.exist?(LODA_RUST_MISMATCHES)
    raise "No such dir #{LODA_RUST_MISMATCHES}, cannot run script"
end

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def cleanup_footer_of_asm_file(path)
    filename = File.basename(path)
    s = IO.read(path)
    modified = false
    100.times do
        # Remove bottom comments and bottom blank lines
        s2 = s.gsub(/^;.*$\n?\z/, '').gsub(/^\s+\z/, '')
        if s == s2
            break
        end
        modified = true
        s = s2
    end
    unless modified
        return 0
    end
    # puts "cleaned up footer of file: #{filename}"
    IO.write(path, s)
    return 1
end

# Identify all the files that are to be renamed
paths = absolute_paths_for_all_programs(LODA_RUST_MISMATCHES)
number_of_programs_analyzed = 0
number_of_programs_that_has_been_cleaned_up = 0
paths.each do |path|
    extension = File.extname(path)
    next unless extension == '.asm'
    number_of_programs_analyzed += 1
    number_of_programs_that_has_been_cleaned_up += cleanup_footer_of_asm_file(path)
end
puts "Total number of programs analyzed: #{number_of_programs_analyzed}"
puts "Total number of programs cleaned up: #{number_of_programs_that_has_been_cleaned_up}"
