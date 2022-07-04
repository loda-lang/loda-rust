#!/usr/bin/env ruby

=begin
This script deletes temporary files from inside the "loda-programs" repo.
The temporary files are used during mining.

The files looks like this:
/Users/johndoe/loda-programs/oeis/004/A004158.asm_benchmark_1653228299999
/Users/johndoe/loda-programs/oeis/004/A004158.asm_check_output_1653228299999
/Users/johndoe/loda-programs/oeis/004/A004158.asm_reject_1653228299999
/Users/johndoe/loda-programs/oeis/005/A005902.asm_benchmark_1653227036629
/Users/johndoe/loda/programs/oeis/314/A314164.asm_original_1653228257240
/Users/johndoe/loda/programs/oeis/025/A025728.asm_deleted_different
=end

require_relative 'config'

LODA_PROGRAMS_REPO = Config.instance.loda_programs_repository
unless File.exist?(LODA_PROGRAMS_REPO)
    raise "No such dir #{LODA_PROGRAMS_REPO}, cannot run script"
end

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
unless File.exist?(LODA_PROGRAMS_OEIS)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

# git: obtain new-files
# https://stackoverflow.com/a/26891150/78336
def absolute_paths_for_unstaged_new_files(dir_inside_repo)
    paths1 = []
    Dir.chdir(dir_inside_repo) do
        result = `git ls-files --exclude-standard --others`
        paths1 = result.split(/\n/)
    end
    paths2 = paths1.map do |path|
        File.join(dir_inside_repo, path)
    end
    paths2
end

unstaged_files = absolute_paths_for_unstaged_new_files(LODA_PROGRAMS_REPO)
#puts "Number of unstaged files: #{unstaged_files.count}"

files_to_be_deleted = []
original_invalid_paths = []
unstaged_files.each do |path|
    filename = File.basename(path)
    if filename =~ /^A\d{6}.asm_original_invalid_\d+$/
        original_invalid_paths << path
        next
    end
    if filename =~ /^A\d{6}.asm_(benchmark|check_output|original|reject)_\d+$/
        files_to_be_deleted << path
        next
    end
    if filename =~ /^A\d{6}.asm_deleted_different$/
        files_to_be_deleted << path
        next
    end
end
puts "Number of files to be deleted from git: #{files_to_be_deleted.count}"
puts "Number of 'original-invalid' files: #{original_invalid_paths.count}, #{original_invalid_paths}"

files_to_be_deleted.each do |path|
    File.delete(path)
end
