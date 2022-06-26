# This script takes a parameter, a path to a CSV file.
#
# The CSV file looks like this. Each of the rows in the CSV file is an absolute path to a program.
# These programs are to be uploaded to loda-lang.org.
#
# input.csv
# /absolute/path/to/loda/programs/oeis/001/A001281.asm
# /absolute/path/to/loda/programs/oeis/047/A047315.asm
# /absolute/path/to/loda/programs/oeis/314/A314736.asm
# /absolute/path/to/loda/programs/oeis/314/A314848.asm
#
# In order to prepare `input.csv`. If have committed files to my local loda-programs repo, and I want to
# upload these files to loda-lang.org, I do like this.
# git diff-tree --no-commit-id --name-only -r 0b98d576e9c562f80a97e37f9487105a4e8a412a > input.csv
# Next I prefix the rows with their absolute paths.
# Finally I run this script with the input.csv file.

require_relative 'upload_program_files_to_server'
require 'csv'

unless ARGV.count == 1
    raise "There must be exactly 1 parameter, the path to a CSV file containing paths to be uploaded"
end
input_csv_path = ARGV[0]
unless File.exist?(input_csv_path)
    raise "No such file #{input_csv_path}, cannot run script"
end

file_paths = []
CSV.foreach(input_csv_path, col_sep: ";") do |row|
    file_paths << row[0]
end

upload_program_files_to_server(file_paths)
