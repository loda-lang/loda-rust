#!/usr/bin/env ruby

=begin

This script takes input from a `caller_callee_list.csv` file, with this format:

    program id;dependency count;program ids
    4;1;4
    5;1;5
    7;1;7
    8;2;8,165190
    10;1;10

This script outputs a `most_called_programs.csv` file, with this format:

    callee program id;dependency count;caller program ids
    4457;1;4458
    4641;2;189687,279620
    4736;4;95891,104568,104572,237587
    4737;6;124258,133823,133824,133825,220073,272900
    4738;2;82693,181176
    4799;1;67989

=end

require 'csv'
require 'set'

input_filename = 'data/caller_callee_list.csv'
output_filename = 'data/most_called_programs.csv'

dict = {}

puts "input: #{input_filename}"
count = 0
CSV.foreach(input_filename, {:col_sep => ";"}) do |col0, col1, col2|
    caller_program_id = col0.to_i
    callee_program_ids = col2.split(',').drop(1)

    callee_program_ids.each do |callee_program_id|
        program_id = callee_program_id.to_i
        set = dict[program_id] || Set.new
        set.add(caller_program_id)
        dict[program_id] = set
    end
    
    count += 1
    #break if count > 100
end

puts "output: #{output_filename}"
sorted_callee_program_ids = dict.keys.sort
CSV.open(output_filename, "wb", {:col_sep => ";"}) do |csv|
    csv << ["callee program id", "dependency count", "caller program ids"]

    sorted_callee_program_ids.each do |callee_program_id|
        caller_program_ids = dict[callee_program_id].to_a.sort
        # puts "#{callee_program_id} => #{caller_program_ids}"
        caller_count = caller_program_ids.count
        pretty_caller_program_ids = caller_program_ids.join(',')
        
        csv << [callee_program_id, caller_count, pretty_caller_program_ids]
    end
end

