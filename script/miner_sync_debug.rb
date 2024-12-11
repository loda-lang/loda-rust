#!/usr/bin/env ruby

# For debugging of the "sync" code
# Intended for LODA developers.
#
# This appends a timestamp to a logfile.
# And it's possible to tweak what is being returned "changed" or "nochange".

require 'time'
require_relative 'config'

timestamp = Time.now.utc.iso8601

log_file_path = File.join(Config.instance.dot_loda_rust, "miner_sync_debug_log.txt")
File.open(log_file_path, "a") do |f| 
    f << "Sync: #{timestamp}\n"
end

#puts "status: nochange"
puts "status: changed"
