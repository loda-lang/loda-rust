#!/usr/bin/env ruby
require_relative 'config'

OUTPUT_DIR = File.expand_path("data/arc_size")

LODA_RUST_EXECUTABLE = Config.instance.loda_rust_executable
unless File.executable?(LODA_RUST_EXECUTABLE)
    raise "No such file #{LODA_RUST_EXECUTABLE}, cannot run script"
end

ARC_REPOSITORY_DATA = Config.instance.arc_repository_data
unless File.directory?(ARC_REPOSITORY_DATA)
    raise "No such dir #{ARC_REPOSITORY_DATA}, cannot run script"
end

if File.directory?(OUTPUT_DIR)
    raise "The OUTPUT_DIR #{OUTPUT_DIR} already exist. Please delete it manually, and try again."
end

Dir.chdir(ARC_REPOSITORY_DATA) do
    paths = Dir.glob("**/*.json")
    #paths = paths.first(5)
    paths.each_with_index do |path, index|
        if index % 100 == 0
            puts "Progress: #{index} of #{paths.count}"
        end
        output_path = File.join(OUTPUT_DIR, path)
        output_dirname = File.dirname(output_path)
        FileUtils.mkdir_p(output_dirname)
        unless File.directory?(output_dirname)
            raise "unable to create dir"
        end
        
        output = `#{LODA_RUST_EXECUTABLE} arc-size #{path}`
        output = output.strip
        #p output
        success = $?.success?
        
        if success 
            #p 'success'
            IO.write(output_path, output)
        else
            puts "could not compute: #{path}"
        end
    end
end

#output = `#{LODA_RUST_EXECUTABLE} arc-size #{program_id}`
#output = output.strip
#success = $?.success?

