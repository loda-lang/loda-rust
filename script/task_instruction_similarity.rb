#!/usr/bin/env ruby

=begin

=end

INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

NUMBER_OF_CLUSTERS = 40
PERCENTAGE_MUST_BE_IDENTICAL = 0.8

class Program
    attr_reader :program_id
    attr_reader :path_input
    attr_reader :path_output
    attr_reader :line_count
    
    def initialize(program_id, path_input, path_output, line_count)
        @program_id = program_id
        @path_input = path_input
        @path_output = path_output
        @line_count = line_count
    end
end

def split_array_into_clusters(ary, cluster_count)
    result = []
    slice_length = (ary.count + cluster_count - 1) / cluster_count
    ary.each_slice(slice_length) do |slice|
        result << slice
    end
    # pad with empty arrays if needed
    while result.count < cluster_count
        result << []
    end
    result
end

def is_similar_programs(current_program, program_of_similar_length)
    line_count0 = current_program.line_count
    line_count1 = program_of_similar_length.line_count
    lc_diff = (line_count0 - line_count1).abs
    if line_count0 < 5 || line_count1 < 5
        if lc_diff > 0
            # puts "skip 0"
            return false
        end
    else
        if line_count0 < 10 && line_count1 < 10
            if lc_diff > 4
                # puts "skip 1"
                return false
            end
        end
    end
    path0 = current_program.path_input
    path1 = program_of_similar_length.path_input
    cmd = "diff --unchanged-group-format='%<' --old-group-format='' --new-group-format='' #{path0} #{path1}"
    #puts "will execute: #{cmd}"
    output = `#{cmd}`
    output.strip!
    if output.empty?
        # puts "skip 2"
        return false
    end
    number_of_identical_lines = output.split("\n").count
    target0 = (line_count0 * PERCENTAGE_MUST_BE_IDENTICAL).ceil
    target1 = (line_count1 * PERCENTAGE_MUST_BE_IDENTICAL).ceil
    if number_of_identical_lines < target0
        # puts "skip 3 #{number_of_identical_lines} < #{target0}"
        return false
    end
    if number_of_identical_lines < target1
        # puts "skip 4 #{number_of_identical_lines} < #{target1}"
        return false
    end
    puts "similar #{current_program.program_id} with #{program_of_similar_length.program_id}. number of lines shared: #{number_of_identical_lines}"
    return true
end

def save_similar_programs(current_program, similar_program_array)
    path = current_program.path_output
    program_ids = similar_program_array.map { |program| program.program_id }
    content = program_ids.join("\n")
    IO.write(path, content)
end

unless File.exist?(INPUT_DIR)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

def count_number_of_lines_in_file(path)
    content = IO.read(path)
    content.split("\n").count
end

rootdir = INPUT_DIR
relative_paths = Dir.glob(File.join("**", "*_instructions.txt"), base: rootdir).sort
#relative_paths = relative_paths.first(10)

# Process all the input files and remember the number of lines
program_ary = []
relative_paths.each_with_index do |relative_path, index|
    relative_path =~ /\bA0*(\d+)_/
    program_id = $1.to_i
    if program_id == 0
        puts "Ignoring invalid program id for relative_path: #{relative_path}"
        next
    end
    path_input = File.join(INPUT_DIR, relative_path)
    output_name = relative_path.gsub('_instructions.txt', '_similar.txt')
    path_output = File.join(OUTPUT_DIR, output_name)
    line_count = count_number_of_lines_in_file(path_input)
    program_ary << Program.new(program_id, path_input, path_output, line_count)
end
#p program_ary

# Sort by program length.
sorted_program_ary = program_ary.sort { |a,b| a.line_count <=> b.line_count }
sorted_program_ary = sorted_program_ary.reverse

# Split into evenly sized clusters
clustered_programs = split_array_into_clusters(sorted_program_ary, NUMBER_OF_CLUSTERS)

# Compare all programs inside the same cluster with each other
program_index = 0
program_count = [sorted_program_ary.count, 10].max
clustered_programs.each_with_index do |programs_of_similar_length, cluster_index|
    programs_of_similar_length.each_with_index do |current_program, current_program_index|
        next if current_program.line_count < 10
        next if current_program.line_count > 30
        
        similar_program_array = []
        programs_of_similar_length.each do |program_of_similar_length|
            if current_program === program_of_similar_length
                next
            end
            # if (program_index % 10000) == 0
            #     puts "program: #{program_index} cluster: #{cluster_index} current_program_index: #{current_program_index}"
            # end
            if is_similar_programs(current_program, program_of_similar_length)
                similar_program_array << program_of_similar_length
            end
        end
        similar_program_array.sort! { |a,b| a.program_id <=> b.program_id }
        
        save_similar_programs(current_program, similar_program_array)
        
        percent = program_index * 100 / program_count
        progress = "#{percent}\% #{program_index}/#{program_count}" 
        puts "#{progress}  #{current_program.program_id} is similar to #{similar_program_array.count} other programs."
        program_index += 1
    end
end


