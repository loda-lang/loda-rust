#!/usr/bin/env ruby
require_relative 'config'
require 'open3'
require 'json'

# Copy an ARC task json file, but erase the output images from the `test` pairs.
#
# The erasing is done, to prevent cheating.
# If the prediction algorithm could look at the output, then it could make awesome predictions.
# Without the output images, then it's harder to cheat.
def copy_task_without_test_output(source_task_json_path, destination_task_json_path)
    if source_task_json_path == destination_task_json_path
        raise "the paths are supposed to be different. #{source_task_json_path}"
    end
    json_string = IO.read(source_task_json_path)
    json = JSON.parse(json_string)
    test_pairs = json['test']
    test_pairs.each do |pair|
        pair['output'] = []
    end
    File.write(destination_task_json_path, JSON.dump(json))
end

# Extract the width/height of all the `test` output images.
#
# Returns an array of strings, example: `["10x14", "14x20", "14x15"]`.
def sizes_from_task(task_json_path)
    json_string = IO.read(task_json_path)
    json = JSON.parse(json_string)
    test_pairs = json['test']
    sizes = []
    test_pairs.each do |pair|
        rows = pair['output']
        columns_min = 255
        columns_max = 0
        rows.each do |row|
            columns_max = [columns_max, row.count].max
            columns_min = [columns_min, row.count].min
        end
        if columns_min != columns_max
            raise "the columns are supposed to have the same length. #{task_json_path}"
        end
        width = columns_min
        height = rows.count
        sizes << "#{width}x#{height}"
    end
    sizes
end

# Extract the predicted width/height of all the `test` output images.
#
# Returns an array of strings, example: `["10x14", "14x20", "14x15"]`.
def predicted_sizes(json_string)
    json = JSON.parse(json_string)
    test_pairs = json['test']
    sizes = []
    test_pairs.each do |pair|
        dict = pair['output_size']
        width = dict['width'].to_i
        height = dict['height'].to_i
        sizes << "#{width}x#{height}"
    end
    sizes
end

OUTPUT_DIR = File.expand_path("data/arc_size")
TEMP_PATH = File.join(OUTPUT_DIR, 'temp.json')

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

FileUtils.mkdir_p(OUTPUT_DIR)
unless File.directory?(OUTPUT_DIR)
    raise "unable to create dir: #{OUTPUT_DIR}"
end

count_tasks = 0
count_ok_predictions = 0
count_bad_predictions = 0
count_cannot_predict = 0
count_other_errors = 0
Dir.chdir(ARC_REPOSITORY_DATA) do
    paths = Dir.glob("**/*.json")

    # Remove json files, that are not ARC tasks.
    paths = paths.reject { |path| File.basename(path) == 'solution_notXORdinary.json' }
    
    paths.each_with_index do |path, index|
        if index % 100 == 0
            puts "Progress: #{index} of #{paths.count}"
        end
        
        # What are the sizes of the output images for the test pairs.
        expected_sizes = sizes_from_task(path)
        
        # Make a copy of the task, but discard the output images for the test pairs.
        copy_task_without_test_output(path, TEMP_PATH)
        
        # Create dirs if needed
        output_path = File.join(OUTPUT_DIR, path)
        output_dirname = File.dirname(output_path)
        FileUtils.mkdir_p(output_dirname)
        unless File.directory?(output_dirname)
            raise "unable to create dir: #{output_dirname}"
        end
        
        # Make predictions about the output sizes
        command = "#{LODA_RUST_EXECUTABLE} arc-size #{TEMP_PATH}"
        stdout_and_stderr, status = Open3.capture2e(command)
        output = stdout_and_stderr
        count_tasks += 1

        unless status.success?
            if output.include?('Cannot predict the output sizes')
                output_path2 = output_path.gsub(/[.]json$/, '-cannot-predict.txt')
                IO.write(output_path2, stdout_and_stderr)
                count_cannot_predict += 1
                next
            else
                output_path2 = output_path.gsub(/[.]json$/, '-error.txt')
                IO.write(output_path2, stdout_and_stderr)
                count_other_errors += 1
                next
            end
        end
        json = stdout_and_stderr.strip
        predicted_sizes = predicted_sizes(json)
        if predicted_sizes != expected_sizes
            #puts "bad prediction: #{predicted_sizes} != #{expected_sizes} for path: #{path}"
            output_path2 = output_path.gsub(/[.]json$/, '-bad-prediction.txt')
            error_message = stdout_and_stderr + "\n\n--\nThis is a bad prediction!\nPredicted #{predicted_sizes}. But the actual size is #{expected_sizes}"
            IO.write(output_path2, error_message)
            count_bad_predictions += 1
            next
        end
        IO.write(output_path, json)
        count_ok_predictions += 1
        next
    end
end

File.delete(TEMP_PATH) if File.exist?(TEMP_PATH)

puts
puts "count_tasks: #{count_tasks}  The number of tasks processed."
puts "count_ok_predictions: #{count_ok_predictions}  Predictions that matches with the actual data."
puts "count_bad_predictions: #{count_bad_predictions}  Predictions that are different than the actual data."
puts "count_cannot_predict: #{count_cannot_predict}  Unable to make a prediction. Insufficient data, lack of algorithms for predicting."
puts "count_other_errors: #{count_other_errors}  Something else went wrong."
