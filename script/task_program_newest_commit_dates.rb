#!/usr/bin/env ruby

=begin
This script traverses all the programs inside the "loda-programs/oeis" dir.
It looks for all the LODA assembly programs there are.
This script determines date the program was most recently modified in the git repository.

This script outputs a `program_modified_dates.csv` file, with this format:

    program id;modified date
    4;1629655855
    5;1649152907
    6;1649694573
    7;1633971417

=end

require 'csv'
require 'date'
require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis

output_filename = 'data/program_modified_dates.csv'

def relative_paths_for_all_programs(rootdir)
    Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
end

def process_files(paths, csv)
    time_start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    progress_n = [(paths.count / 1000), 1].max
    number_of_rows = 0
    paths.each_with_index do |path, index|
        if (index % progress_n) == 0
            percent = (100 * index).to_f / paths.count
            percent_s = "%.2f" % percent
            
            time_end = Process.clock_gettime(Process::CLOCK_MONOTONIC)
            time_elapsed = time_end - time_start
            time_elapsed_s = "%.3f" % time_elapsed
            
            puts "progress: #{index}/#{paths.count}, %#{percent_s}  rows: #{number_of_rows}  elapsed: #{time_elapsed_s}"
        end
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        
        # Get unit timestamp for newest "commit date" of a file in a repository.
        # https://stackoverflow.com/questions/8611486/how-to-get-the-last-commit-date-for-a-bunch-of-files-in-git
        output = `git log -n 1 --format=%ct -- #{path}`
        # The output looks like this: "1649694573"
        timestamp_string = output.strip
        success = $?.success?
        if !success
            puts "Failure obtaining git commit date for path: #{path}"
            next 
        end
        if timestamp_string.empty?
            puts "Empty output from git log for path: #{path}"
            next 
        end
        unless timestamp_string =~ /^\d+$/
            puts "malformed output from git log for path: #{path}"
            next
        end
        
        csv << [program_id, timestamp_string]
        csv.flush
        number_of_rows += 1
    end
    puts "number of rows written to csv file: #{number_of_rows}"
end

paths = relative_paths_for_all_programs(LODA_PROGRAMS_OEIS)
CSV.open(output_filename, "wb", col_sep: ";") do |csv|
    csv << ["program id", "modified date"]
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        process_files(paths, csv)
    end
end
