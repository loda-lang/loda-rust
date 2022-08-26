#!/usr/bin/env ruby

=begin
Traverse all the unstaged .asm files in the "loda-programs" repository.

Insert a header, like this:
; A123456: Oeis name
; Submitted by John Doe

When encountering a `seq` instruction, then insert the corresponding oeis name.
=end

require 'set'
require_relative 'config'

LODA_PROGRAMS_REPO = Config.instance.loda_programs_repository
unless File.exist?(LODA_PROGRAMS_REPO)
    raise "No such dir #{LODA_PROGRAMS_REPO}, cannot run script"
end

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
unless File.exist?(LODA_PROGRAMS_OEIS)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

OEIS_NAMES_FILE = Config.instance.oeis_names_file
unless File.exist?(OEIS_NAMES_FILE)
    raise "No such file #{OEIS_NAMES_FILE}, cannot run script"
end

LODA_SUBMITTED_BY = Config.instance.loda_submitted_by

# git: obtain modified-files and new-file
# https://stackoverflow.com/a/26891150/78336
def absolute_paths_for_unstaged_files(dir_inside_repo)
    paths1 = []
    Dir.chdir(dir_inside_repo) do
        result = `git ls-files --exclude-standard --modified --others`
        paths1 = result.split(/\n/)
    end
    paths2 = paths1.map do |path|
        File.join(dir_inside_repo, path)
    end
    paths2
end

def absolute_paths_for_unstaged_programs_that_exist
    paths1 = absolute_paths_for_unstaged_files(LODA_PROGRAMS_OEIS)
    paths2 = paths1.filter { |path| File.exist?(path) }
    paths3 = paths2.filter { |path| path =~ /[.]asm$/ }
    paths4 = paths3.filter { |path| path =~ /\boeis\b/ }
    paths4
end

def extract_oeis_ids_from_program_file(path)
    unless File.exist?(path)
        raise "file does not exist: #{path}"
    end
    path =~ /\bA0*(\d+)[.]asm$/
    program_id = $1.to_i
    if program_id == 0
        raise "Unable to process file at path: #{path}"
    end
    content = IO.read(path)
    sequence_instructions = content.scan(/^\s*seq .*,\s*(\d+)$/).flatten
    sequence_program_ids = sequence_instructions.map { |seq_program_id| seq_program_id.to_i }
    # puts "program: #{program_id} depends on: #{sequence_program_ids}"
    return [program_id] + sequence_program_ids
end

def extract_oeis_ids_from_program_files(paths)
    program_ids = []
    paths.each do |path|
        program_ids += extract_oeis_ids_from_program_file(path)
    end
    program_ids
end

def read_original_file_from_repo(path)
    dir_inside_repo = LODA_PROGRAMS_REPO
    file_content = nil
    Dir.chdir(dir_inside_repo) do
        path_relative_to_repo_root = `git ls-files --full-name #{path}`
        path_relative_to_repo_root.strip!
        file_content = `git show HEAD:#{path_relative_to_repo_root}`
    end
    file_content
end

def update_names_in_program_file(path, oeis_name_dict, loda_submitted_by)
    unless File.exist?(path)
        raise "file does not exist: #{path}"
    end
    path =~ /\b(A0*(\d+))[.]asm$/
    oeis_id = $1
    program_id = $2.to_i
    if program_id == 0
        raise "Unable to process file at path: #{path}"
    end
    program_name = oeis_name_dict[program_id]

    content = IO.read(path).strip
    # Get rid of top comments and blank lines
    content.gsub!(/^;.*\n/, '')
    content.gsub!(/^\s*\n+/, "")

    # extract terms from the original file in git
    # in order to keep the noise as low as possible when diff'ing the files.
    # if the length changes between old/new files in git, then it's time consuming to verify.
    original_content = read_original_file_from_repo(path)
    original_terms_comment = ""
    original_content.scan(/^;\s*-?\d+\s*,\s*-?\d+.*$/) do |match|
        original_terms_comment = match.to_s
        break
    end
    
    terms_comment = original_terms_comment
    if terms_comment.empty?
        terms = `loda eval #{oeis_id} -t 60`.strip
        terms_comment = "; #{terms}"
    end
    
    # Identify with `seq` instructions, and insert their corresponding sequence name.
    content.gsub!(/^\s*seq .*,\s*(\d+)$/) do |match|
        sequence_program_id = $1.to_i
        sequence_name = oeis_name_dict[sequence_program_id]
        "#{match} ; #{sequence_name}"
    end
    
    new_content = ""
    new_content += "; #{oeis_id}: #{program_name}\n"
    new_content += "; Submitted by #{loda_submitted_by}\n"
    new_content += terms_comment + "\n\n"
    new_content += content
    new_content += "\n"
    
    IO.write(path, new_content)
    puts "Updated program: #{path}"
end

def update_names_in_program_files(paths, oeis_name_dict, loda_submitted_by)
    paths.each do |path|
        update_names_in_program_file(path, oeis_name_dict, loda_submitted_by)
    end
end

paths = absolute_paths_for_unstaged_programs_that_exist
#p paths
if paths.empty?
    puts "There are no unstaged .asm files in this repository."
    exit 0
end

puts "Number of unstaged .asm files: #{paths.count}"

# Harvest all the program ids that needs to have their name looked up
program_ids = extract_oeis_ids_from_program_files(paths)
program_ids_set = program_ids.to_set
puts "Will lookup names for these program ids: #{program_ids_set.to_a.sort}" 

oeis_name_dict = {}
approx_row_count = 400000
File.new(OEIS_NAMES_FILE, "r").each_with_index do |line, index|
    if (index % 30000) == 0
        percentage = (100 * index) / approx_row_count
        puts "progress %#{percentage}  #{index}/#{approx_row_count}"
    end
    next unless line =~ /^A0*(\d+) (.+)$/
    program_id = $1.to_i
    name = $2
    next unless program_ids_set.include?(program_id)
    #puts "program_id: #{program_id} name: #{name}"
    oeis_name_dict[program_id] = name
end

oeis_name_dict_set = oeis_name_dict.keys.to_set

if oeis_name_dict.count != program_ids_set.count
    puts "oeis_name_dict.count=#{oeis_name_dict.count} != program_ids_set.count=#{program_ids_set.count}"
    puts "oeis_name_dict: #{oeis_name_dict}"
    puts "program_ids_set: #{program_ids_set}"
    common_set = oeis_name_dict_set & program_ids_set
    difference_set = (oeis_name_dict_set - common_set) + (program_ids_set - common_set)
    puts "difference_set: #{difference_set}"
    raise "Inconsistency: Unable to lookup all program_ids. Please run `loda mine` to fetch the latest OEIS 'stripped' file."
end

update_names_in_program_files(paths, oeis_name_dict, LODA_SUBMITTED_BY)
