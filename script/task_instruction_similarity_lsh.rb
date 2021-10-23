#!/usr/bin/env ruby

=begin
My second attempt at identifying similar programs.
=end

require 'csv'
require 'set'

INPUT_FILE_BIGRAM = 'data/bigram.csv'
INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

SIGNATURE_LENGTH = 10
NUMBER_OF_PROGRESS_PRINTS = 50
NUMBER_OF_CLUSTERS = 40
PERCENTAGE_MUST_BE_IDENTICAL = 0.8

unless File.exist?(INPUT_DIR)
    raise "No such dir #{LODA_PROGRAMS_OEIS}, cannot run script"
end

unless File.exist?(INPUT_FILE_BIGRAM)
    raise "No such file #{INPUT_FILE_BIGRAM}, cannot run script"
end

class Program
    attr_reader :program_id
    attr_reader :path_input
    attr_reader :path_output
    attr_reader :line_count
    attr_reader :signature
    
    def initialize(program_id, path_input, path_output, line_count, signature)
        @program_id = program_id
        @path_input = path_input
        @path_output = path_output
        @line_count = line_count
        @signature = signature
    end
end

class ComparisonResult
    attr_reader :program_id
    attr_reader :a_line_count
    attr_reader :b_line_count
    attr_reader :overlap_line_count
    
    def initialize(program_id, a_line_count, b_line_count, overlap_line_count)
        @program_id = program_id
        @a_line_count = a_line_count
        @b_line_count = b_line_count
        @overlap_line_count = overlap_line_count
    end
    
    def jaccard_index
        x = @overlap_line_count
        y = @a_line_count + @b_line_count - @overlap_line_count
        x.to_f / y.to_f
    end
    
    def human_readable_jaccard_index
        "%.4f" % jaccard_index
    end
end

def compare_programs(program0, program1)
    line_count0 = program0.line_count
    line_count1 = program1.line_count
    lc_diff = (line_count0 - line_count1).abs
    if line_count0 < 5 || line_count1 < 5
        if lc_diff > 0
            # puts "skip 0"
            return nil
        end
    else
        if line_count0 < 10 && line_count1 < 10
            if lc_diff > 4
                # puts "skip 1"
                return nil
            end
        end
    end
    path0 = program0.path_input
    path1 = program1.path_input
    cmd = "diff --unchanged-group-format='%<' --old-group-format='' --new-group-format='' #{path0} #{path1}"
    #puts "will execute: #{cmd}"
    output = `#{cmd}`
    output.strip!
    if output.empty?
        # puts "skip 2"
        return nil
    end
    number_of_identical_lines = output.split("\n").count
    target = (line_count0 * PERCENTAGE_MUST_BE_IDENTICAL).ceil
    if number_of_identical_lines < target
        # puts "skip 3 #{number_of_identical_lines} < #{target}"
        return nil
    end
    puts "similar #{program0.program_id} with #{program1.program_id}. number of lines shared: #{number_of_identical_lines}"
    return ComparisonResult.new(program1.program_id, line_count0, line_count1, number_of_identical_lines)
end

def save_similar_programs(current_program, comparison_result_array)
    path = current_program.path_output
    CSV.open(path, "wb", col_sep: ";") do |csv|
        csv << ["program_id", "overlap_count", "jaccard_index"]
        comparison_result_array.each_with_index do |comparison_result, index|
            row = [
                comparison_result.program_id,
                comparison_result.overlap_line_count,
                comparison_result.human_readable_jaccard_index
            ]
            csv << row
            # break if index == 10
        end
    end
end

def load_bigram(path)
    vocabulary = []
    CSV.foreach(path, col_sep: ";") do |row|
        col0 = row[0]
        count = col0.to_i
        next if count == 0
        word0 = row[1]
        word1 = row[2]
        vocabulary << "#{word0}#{word1}"
    end
    vocabulary.sort
end

def signature_and_line_count_program(path, vocabulary, indexes_array)
    # puts "processing #{path}"
    content = IO.read(path)
    line_array = content.split("\n")
    line_count = line_array.count
    line_array2 = ['START'] + line_array + ['STOP']
    # p line_array2

    match_set = Set.new
    line_array2.each_cons(2) do |word0, word1|
        bigram = "#{word0}#{word1}"
        index = vocabulary.index(bigram)
        # puts "#{bigram} #{index}"
        match_set << index
    end
    # p match_set
    matches = match_set.to_a.sort
    # p matches

    signature = []
    indexes_array.each do |indexes|
        signature_item = -1
        # stop at first matching hash
        skip_count = 0
        indexes.each_index do |index|
            j = indexes.index(index)
            if !match_set.include?(j)
                # puts "skip #{j}"
                skip_count += 1
                next
            end
            # puts "matched: #{j}, #{match_set}  skip_count: #{skip_count}"
            signature_item = j
            break
        end
        signature << signature_item
    end
    # p signature
    [signature, line_count]
end

vocabulary = load_bigram(INPUT_FILE_BIGRAM)
puts "vocabulary size: #{vocabulary.count}"
#p vocabulary

vocabulary_indexes = (0..vocabulary.count-1).to_a
#p vocabulary_indexes

# Create permutations of the numbers between (0 .. vocabulary.count-1)
indexes_array = []
SIGNATURE_LENGTH.times do |i|
    seed = 10 * i
    indexes = vocabulary_indexes.shuffle(random: Random.new(seed))
    indexes_array << indexes
end
#p indexes_array

rootdir = INPUT_DIR
relative_paths = Dir.glob(File.join("**", "*_instructions.txt"), base: rootdir).sort
#relative_paths = relative_paths.first(10)

# Process all the input files, and create a signature
program_ary = []
row_count = relative_paths.count
row_count_mod = (row_count / NUMBER_OF_PROGRESS_PRINTS).ceil
t0 = Time.now
number_of_too_short_programs = 0
number_of_too_long_programs = 0
relative_paths.each_with_index do |relative_path, index|
    if (index % row_count_mod) == 0
        percentage = (100 * index) / row_count
        puts "progress %#{percentage}  #{index}/#{row_count}"
    end

    relative_path =~ /\bA0*(\d+)_/
    program_id = $1.to_i
    if program_id == 0
        puts "Ignoring invalid program id for relative_path: #{relative_path}"
        next
    end
    path_input = File.join(INPUT_DIR, relative_path)
    output_name = relative_path.gsub('_instructions.txt', '_similarity2.csv')
    path_output = File.join(OUTPUT_DIR, output_name)
    signature, line_count = signature_and_line_count_program(path_input, vocabulary, indexes_array)
    if line_count < 4
        # puts "Ignoring too short program: #{relative_path}"
        number_of_too_short_programs += 1
        next
    end
    if line_count > 60
        # puts "Ignoring too long program: #{relative_path}"
        number_of_too_long_programs += 1
        next
    end
    program_ary << Program.new(program_id, path_input, path_output, line_count, signature)
    if program_ary.count >= 500
        break
    end
end
t1 = Time.now
elapsed = t1 - t0
puts "number of ignored programs. too short: #{number_of_too_short_programs}  too long: #{number_of_too_long_programs}"
puts "computed signatures for #{program_ary.count} programs. Elapsed: #{elapsed}"

def compare_signature(program0, program1)
    signature0 = program0.signature
    signature1 = program1.signature
    if signature0.count != signature1.count
        return 0
    end
    number_of_matches = 0
    signature0.zip(signature1).each do |signature0_item, signature1_item|
        if signature0_item == signature1_item
            number_of_matches += 1
        end
    end
    jaccard_index = number_of_matches.to_f / signature0.count
    return jaccard_index
end

program_ary = program_ary.first(50)

number_of_mismatches = 0
number_of_matches = 0
program_ary.each do |program0|
    program_ary.each do |program1|
        next if program0 === program1
        jaccard_index = compare_signature(program0, program1)
        if jaccard_index < 0.25
            number_of_mismatches += 1
            next
        end
        # puts "#{program0.program_id} #{program1.program_id} #{jaccard_index}"
        number_of_matches += 1
    end
end

puts "number_of_matches: #{number_of_matches}"
puts "number_of_mismatches: #{number_of_mismatches}"






