# Identifies the added/changed programs in latest commit within the loda-programs repo, and submits these programs to loda-lang.org.

require_relative 'config'
require_relative 'upload_program_files_to_server'

LODA_PROGRAMS_REPO = Config.instance.loda_programs_repository
unless File.exist?(LODA_PROGRAMS_REPO)
    raise "No such dir #{LODA_PROGRAMS_REPO}, cannot run script"
end

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
unless File.exist?(LODA_PROGRAMS_OEIS)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

def absolute_paths_for_files_to_be_uploaded(dir_inside_repo)
    paths1 = []
    Dir.chdir(dir_inside_repo) do
        result = `git diff-tree --no-commit-id --name-only -r HEAD`
        paths1 = result.split(/\n/)
    end
    paths2 = paths1.map do |path|
        File.join(dir_inside_repo, path)
    end
    paths2
end

file_paths = absolute_paths_for_files_to_be_uploaded(LODA_PROGRAMS_REPO)
upload_program_files_to_server(file_paths)
