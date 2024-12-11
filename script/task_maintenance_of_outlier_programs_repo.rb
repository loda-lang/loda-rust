#!/usr/bin/env ruby

=begin
Run "maintenance" job inside the "loda-outlier-programs" repository.
=end

require_relative 'config'

LODA_OUTLIER_PROGRAMS_DIR = Config.instance.loda_outlier_programs_repository
unless Dir.exist?(LODA_OUTLIER_PROGRAMS_DIR)
    raise "No such dir #{LODA_OUTLIER_PROGRAMS_DIR}, cannot run script"
end

script_path = File.join(LODA_OUTLIER_PROGRAMS_DIR, "script")
unless Dir.exist?(script_path)
    raise "No such dir #{script_path}, cannot run script"
end

Dir.chdir(script_path) do
    system("rake maintenance")
end
