#!/usr/bin/env ruby

=begin

This script generates an image with an overview of what files already exist and
what is yet to be mined.

This script traverses all the programs inside the LODA program rootdir.

For every program that exist, a black pixel is plotted.
White pixels when there exist no program.

The image width is always 1000 pixels.
The image height varies depending on how many programs that have been mined.
The top-left pixel correspond to A000000.
The top-right pixel correspond to A000999.

=end

require 'set'
require_relative 'config'

LODA_PROGRAM_ROOTDIR = Config.instance.loda_program_rootdir

def obtain_program_ids(rootdir)
    paths = Dir.glob(File.join("**", "*.asm"), base: rootdir)

    program_ids = Set.new
    paths.map do |path|
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        program_ids.add(program_id)
    end
    program_ids
end

def generate_image(program_ids)
    # p program_ids.count
    highest_program_id = program_ids.max
    # p highest_program_id
    
    image_width = 1000
    image_height = (highest_program_id / 1000) + 1
    # p image_height
    
    rows = []
    rows << "P1"
    rows << "\# loda_file_status_image.pbm"
    rows << "#{image_width} #{image_height}"
    image_height.times do |y|
        row = []
        image_width.times do |x|
            offset = y * image_width + x
            program_exist = program_ids.include?(offset)
            if program_exist
               row << 1
            else 
               row << 0
            end
        end
        rows << row.join(' ')
    end
    filename = "data/loda_file_status_image.pbm"
    content = rows.join("\n")
    IO.write(filename, content)
    puts "generated file: '#{filename}'  filesize: #{content.bytes.count}"
end

program_ids = obtain_program_ids(LODA_PROGRAM_ROOTDIR)
generate_image(program_ids)

