#!/usr/bin/env ruby

=begin
This script scans the "loda-programs" repo for newly mined programs, or programs that have been updated with a better version.
These files gets added to the repo.
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

unstaged_files = absolute_paths_for_unstaged_files(LODA_PROGRAMS_REPO)
puts "Number of unstaged files: #{unstaged_files.count}"

files_to_be_added = []
unstaged_files.each do |path|
    filename = File.basename(path)
    if filename =~ /^A\d{6}.asm$/
        files_to_be_added << path
    end
end
puts "Number of files to be added to git: #{files_to_be_added.count}"

Dir.chdir(LODA_PROGRAMS_REPO) do
    files_to_be_added.each do |path|
        `git add #{path}`
    end
end
