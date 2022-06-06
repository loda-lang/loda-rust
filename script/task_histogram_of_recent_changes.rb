#!/usr/bin/env ruby

=begin

This script takes input from a `program_modified_dates.csv` file, with this format:

    program id;modified date
    2;1654106907
    4;1629655855
    5;1653847953
    6;1649694573
    7;1633971417

This script extract the modified date column, and makes a histogram.

=end

require 'csv'
require 'date'

INPUT_FILENAME = 'data/program_modified_dates.csv'

# Extract the dates
dates = []
CSV.foreach(INPUT_FILENAME, col_sep: ";") do |row|
    col0 = row[0]
    col1 = row[1]
    program_id = col0.to_i
    next if program_id == 0

    date = DateTime.strptime(col1,'%s')
    dates << date
end

newest_date = dates.max
oldest_date = dates.min
number_of_days = (newest_date - oldest_date).ceil
puts "number_of_days: #{number_of_days}  (date range: #{oldest_date}..#{newest_date})"

histogram_day = []
number_of_days.times do |day_index|
    count = 0
    
    t0 = oldest_date + day_index
    t1 = t0 + 1
    
    dates.each do |date|
        if date >= t0 && date < t1
            count += 1
        end
    end
    if count > 2000
        puts "histogram_day: clamped day_index #{day_index} count: #{count} to 2000"
        count = 2000
    end
    
    histogram_day << count
end
puts "day histogram"
p histogram_day

histogram_week = []
((number_of_days + 7) / 7).times do |day_index|
    count = 0
    
    t0 = oldest_date + (day_index * 7)
    t1 = t0 + 7
    
    dates.each do |date|
        if date >= t0 && date < t1
            count += 1
        end
    end
    if count > 10000
        puts "histogram_week: clamped day_index #{day_index} count: #{count} to 10000"
        count = 10000
    end
    
    histogram_week << count
end
puts "week histogram"
p histogram_week
