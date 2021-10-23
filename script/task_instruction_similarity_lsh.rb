#!/usr/bin/env ruby

=begin
My second attempt at identifying similar programs.
=end

require 'csv'
require 'set'

INPUT_FILE_BIGRAM = 'data/bigram.csv'
INPUT_DIR = 'data/instructions'
OUTPUT_DIR = 'data/instructions'

SIGNATURE_LENGTH = 20
OVERLAP_MATCH_LIMIT = 10
NUMBER_OF_PROGRESS_PRINTS = 50

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
    attr_reader :overlap_count
    attr_reader :signature_length
    
    def initialize(program_id, overlap_count, signature_length)
        @program_id = program_id
        @overlap_count = overlap_count
        @signature_length = signature_length
    end
    
    def jaccard_index
        x = @overlap_count
        y = @signature_length
        x.to_f / y.to_f
    end
    
    def human_readable_jaccard_index
        "%.4f" % jaccard_index
    end
end

def save_similar_programs(current_program, comparison_result_array)
    path = current_program.path_output
    CSV.open(path, "wb", col_sep: ";") do |csv|
        csv << ["program_id", "overlap_count", "signature_length", "jaccard_index"]
        comparison_result_array.each_with_index do |comparison_result, index|
            row = [
                comparison_result.program_id,
                comparison_result.overlap_count,
                comparison_result.signature_length,
                comparison_result.human_readable_jaccard_index
            ]
            csv << row
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

    signature = 0
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
        binary = 1 << signature_item
        signature |= binary
    end
    # p signature
    [signature, line_count]
end

def process_all_input_files(vocabulary, indexes_array)
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
        output_name = relative_path.gsub('_instructions.txt', '_similarity_lsh.csv')
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
        # if program_ary.count >= 5000
        #     break
        # end
    end
    t1 = Time.now
    elapsed = t1 - t0
    puts "number of ignored programs. too short: #{number_of_too_short_programs}  too long: #{number_of_too_long_programs}"
    puts "computed signatures for #{program_ary.count} programs. Elapsed: #{elapsed}"
    program_ary
end

def compare_signature(program0, program1)
    signature0 = program0.signature
    signature1 = program1.signature
    overlap = signature0 & signature1
    # puts "signature0: #{signature0.to_s(2)}"
    # puts "signature1: #{signature1.to_s(2)}"
    # puts "overlap:    #{overlap.to_s(2)}"
    if overlap == 0
        return 0
    end
    overlap_count = overlap.to_s(2).count("1")
    # puts "overlap_count: #{overlap_count}"
    return overlap_count
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

program_ary = process_all_input_files(vocabulary, indexes_array)

# p program_ary.first.signature.to_s(2)
# raise

#program_ary = program_ary.first(50)

number_of_mismatches = 0
number_of_matches = 0
row_count = program_ary.count
row_count_mod = (row_count / NUMBER_OF_PROGRESS_PRINTS).ceil
t0 = Time.now
program_ary.each_with_index do |program0, program0_index|
    percent = (100 * program0_index) / row_count
    progress = "#{percent}\% #{program0_index}/#{row_count}"
    if (program0_index % row_count_mod) == 0
        match_ratio = number_of_matches.to_f / (number_of_matches + number_of_mismatches)
        t1 = Time.now
        elapsed = t1 - t0
        puts "PROGRESS: #{progress}  matches: #{number_of_matches} mismatches: #{number_of_mismatches}  ratio: #{match_ratio}  elapsed: #{elapsed}"
    end
    comparison_result_array = []
    program_ary.each do |program1|
        next if program0 === program1
        overlap_count = compare_signature(program0, program1)
        if overlap_count < OVERLAP_MATCH_LIMIT
            number_of_mismatches += 1
            next
        end
        number_of_matches += 1
        comparison_result = ComparisonResult.new(program1.program_id, overlap_count, SIGNATURE_LENGTH)
        comparison_result_array << comparison_result
    end
    comparison_result_array.sort! { |a,b| a.program_id <=> b.program_id }
    
    save_similar_programs(program0, comparison_result_array)
    puts "#{progress}  #{program0.program_id} is similar to #{comparison_result_array.count} other programs."
end
t1 = Time.now
elapsed = t1 - t0
puts "comparing signatures. elapsed #{elapsed}"
puts "number_of_matches: #{number_of_matches}"
puts "number_of_mismatches: #{number_of_mismatches}"
match_ratio = number_of_matches.to_f / (number_of_matches + number_of_mismatches)
puts "match_ratio: #{match_ratio}"

