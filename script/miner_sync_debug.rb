#!/usr/bin/env ruby
require 'time'
require_relative 'config'

timestamp = Time.now.utc.iso8601

log_file_path = File.join(Config.instance.dot_loda_rust, "miner_sync_debug_log.txt")
File.open(log_file_path, "a") do |f| 
    f << "Sync: #{timestamp}\n"
end
