#!/usr/bin/env ruby

=begin

This script takes two input files.

This script takes input from a `bigram.csv` file, with this format:

    count;word0;word1
    18066;mov;mov
    16888;START;mov
    14712;mov;lpb
    13386;mov;sub
    13132;mov;add
    11776;add;mov
    10522;add;add
    9840;mul;add

This script takes input from a `program_ids.csv` file, with this format:

    program id
    4
    5
    7
    8
    10

This script prints something ala:

    START
    mov $1,$1
    pow $0,$0
    add $0,$1
    cal $0,35008
    mul $0,$1
    add $0,$0
    add $1,1
    STOP

=end

require 'csv'

input_filename0 = 'data/bigram.csv'
input_filename1 = 'data/program_ids.csv'

random_seed = 1049
max_program_length = 10

# Weighted random sampling
def pick_random_index(weights_array, weights_total, random_generator)
    target_weight = random_generator.rand(weights_total)
    sum = 0
    weights_array.each_with_index do |weight, index|
        sum += weight
        if sum >= target_weight
            return index
        end
    end
    0
end

instruction_to_parameters = {
    "add" => ['a', 'b-add'], 
    "bin" => ['a', 'b'], 
    "cal" => ['a', 'b-cal'], 
    "clr" => ['a', 'b-clr'], 
    "cmp" => ['a', 'b'], 
    "dif" => ['a', 'b-dif'], 
    "div" => ['a', 'b-div'], 
    "gcd" => ['a', 'b-gcd'], 
    "log" => ['a', 'b-log'], 
    "lpb" => ['a', 'b-lpb'], 
    "lpe" => [], 
    "max" => ['a', 'b'], 
    "min" => ['a', 'b'], 
    "mod" => ['a', 'b-mod'], 
    "mov" => ['a', 'b-mov'], 
    "mul" => ['a', 'b-mul'], 
    "pow" => ['a', 'b'], 
    "sub" => ['a', 'b-sub'], 
    "trn" => ['a', 'b'],
    "START" => [],
    "STOP"  => [],
}

def pick_random_parameter_data(parameter_spec, program_ids, random_generator)
    case parameter_spec
    when 'a'
        x = random_generator.rand(2)
        return '$' + x.to_s
    when 'b'
        x = random_generator.rand(2)
        y = random_generator.rand(3)
        prefix = ''
        if y == 0
            prefix = '$'
        end
        return prefix + x.to_s
    when 'b-add'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(4) + 1
            return x.to_s
        end
    when 'b-cal'
        x = random_generator.rand(program_ids.count)
        y = program_ids[x]
        return y.to_s
    when 'b-clr'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(4) + 1
            return x.to_s
        end
    when 'b-dif'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(8) + 2
            return x.to_s
        end
    when 'b-div'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(8) + 2
            return x.to_s
        end
    when 'b-gcd'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(8) + 2
            return x.to_s
        end
    when 'b-log'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(8) + 2
            return x.to_s
        end
    when 'b-lpb'
        x = random_generator.rand(2) + 1
        return x.to_s
    when 'b-mod'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(9) + 1
            return x.to_s
        end
    when 'b-mov'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(9) + 1
            return x.to_s
        end
    when 'b-mul'
        y = random_generator.rand(3)
        case y
        when 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        when 1
            x = random_generator.rand(8) + 2
            return x.to_s
        else
            return '-1'
        end
    when 'b-sub'
        if random_generator.rand(2) == 0
            x = random_generator.rand(3)
            return '$' + x.to_s
        else
            x = random_generator.rand(9) + 1
            return x.to_s
        end
    else
        return '0'
    end
end



# Load the CSV file
bigram_rows = []
CSV.foreach(input_filename0, col_sep: ";") do |row|
    col0, col1, col2 = row
    count = col0.to_i
    next if count == 0
    bigram_rows << [count, col1, col2]
end
# puts "bigram_rows: #{bigram_rows.count}"
# p bigram_rows.first(20)


# Obtain all the program_ids
program_ids = []
CSV.foreach(input_filename1, col_sep: ";") do |row|
    col0 = row[0]
    program_id = col0.to_i
    next if program_id == 0
    program_ids << program_id
end
# puts "program_ids: #{program_ids.count}"


# Create a mapping from the previous word to the most likely next word
dict_word = {}
dict_weight = {}
dict_weighttotal = {}
bigram_rows.each do |bigram_row|
    weight, word0, word1 = bigram_row
    
    ary0 = dict_word[word0] || []
    ary0 << word1
    dict_word[word0] = ary0
    
    ary = dict_weight[word0] || []
    ary << weight
    dict_weight[word0] = ary
    
    sum = dict_weighttotal[word0] || 0
    sum += weight
    dict_weighttotal[word0] = sum
end
# puts "dict_word: #{dict_word.count}  dict_weight: #{dict_weight.count}  dict_weighttotal: #{dict_weighttotal.count}"
# p dict_word
# p dict_weight
# p dict_weighttotal

# Convert the dictionary to an array, so that we can use integers as keys
reserved_words = ['START', 'STOP']
word0_array = (reserved_words + dict_word.keys.to_a + dict_weight.keys.to_a + dict_weighttotal.keys.to_a).uniq
word0_to_word_array = []
word0_to_weight_array = []
word0_weight = []
word0_array.each do |word|
    word0_to_word_array << dict_word[word]
    word0_to_weight_array << dict_weight[word]
    word0_weight << dict_weighttotal[word]
end
# p word0_array
# p word0_to_word_array
# p word0_to_weight_array
# p word0_weight


random = Random.new(random_seed)

program_instructions = ['START']

# Append random instructions
(max_program_length-1).times do |i|
    word0 = program_instructions.last
    index = word0_array.index(word0)

    # stop when reaching the STOP token
    if word0 == 'STOP'
        break
    end

    word_array = word0_to_word_array[index]
    weight_array = word0_to_weight_array[index]
    weight_sum = word0_weight[index]
    
    next_index = pick_random_index(weight_array, weight_sum, random)
    next_word = word_array[next_index]
    program_instructions << next_word
end

#p program_instructions

# Extend the instructions with random parameters
program_instructions.each do |instruction|
    # puts "#{instruction}"

    parameters = instruction_to_parameters[instruction]
    if parameters == nil
        raise "encountered an unknown instruction #{instruction}"
    end

    params = []
    parameters.each do |parameter|
        params << pick_random_parameter_data(parameter, program_ids, random)
    end
    
    row = instruction
    if !params.empty?
        row += ' ' + params.join(',')
    end
    puts row
end
