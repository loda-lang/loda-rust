#!/usr/bin/env ruby

=begin

Proposal: Instruction "zap" for removal of a divisor
https://github.com/loda-lang/loda-cpp/issues/182

This script traverses all the programs inside the "loda-programs/oeis" dir.
It counts the number of occurences of this snippet
lpb $0
  dif $0,2
lpe

As of 2022-oct-30 there are 548 places, that can be replaced by a `zap` instruction.

=end

require 'csv'
require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis

def absolute_paths_for_all_programs(rootdir)
    relative_paths = Dir.glob(File.join("**", "*.asm"), base: rootdir).sort
    absolute_paths = relative_paths.map { |relative_path| File.join(rootdir, relative_path) }
    absolute_paths
end

def process_paths(paths)
    total = 0
    paths.map do |path|
        path =~ /0*(\d+)[.]asm/
        program_id = $1.to_i
        if program_id == 0
            puts "Mismatch for #{filename}"
            next
        end
        content = IO.read(path)
        n = content.scan(/lpb [$]\d+\n\s*dif [$]\d+,[$]?\d+\n\s*lpe/m).count
        if n == 0
            next
        end
        total += 1
        if (total % 50) == 0
            puts "#{total}"
        end
    end
    puts "total: #{total}"
end

paths = absolute_paths_for_all_programs(LODA_PROGRAMS_OEIS)
process_paths(paths)
