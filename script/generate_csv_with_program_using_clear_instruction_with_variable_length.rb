#!/usr/bin/env ruby

=begin

This script traverses all the programs inside the LODA program rootdir.
It looks for all the LODA assembly programs there are.
When encountering a program that contains a problematic instruction, then it's appended to the CSV file.

This script outputs a `programs_using_clear_instruction_with_variable_length.csv` file, with this format:

    program id
    655
    1823
    20821
    26474
    34214

=end

require 'csv'
require_relative 'config'

LODA_PROGRAM_ROOTDIR = Config.instance.loda_program_rootdir

output_filename = 'programs_using_clear_instruction_with_variable_length.csv'

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def obtain_info_rows(paths)
    info_rows = []
    paths.map do |path|
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        content = IO.read(path)
        n = content.scan(/clr [$]\d+,[$]/).count
        if n == 0
            next
        end
        info_row = [program_id]
        info_rows << info_row
    end
    info_rows
end

paths = absolute_paths_for_all_programs(LODA_PROGRAM_ROOTDIR)
info_rows = obtain_info_rows(paths)
puts "count: #{info_rows.count}"

CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["program id"]
    info_rows.each_with_index do |info_row, index|
        csv << info_row
        # break if index == 10
    end
end
