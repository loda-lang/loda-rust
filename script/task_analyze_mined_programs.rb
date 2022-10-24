#!/usr/bin/env ruby

=begin
This traverses all the candidate programs inside the "mine-event" dir.
It decides whether to keep or reject the programs.
=end

require_relative 'config'

LODA_RUST_EXECUTABLE = Config.instance.loda_rust_executable
unless File.executable?(LODA_RUST_EXECUTABLE)
    raise "No such file #{LODA_RUST_EXECUTABLE}, cannot run script"
end

exec("#{LODA_RUST_EXECUTABLE} postmine")
