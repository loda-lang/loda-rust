#!/usr/bin/env ruby

=begin

This script compiles the `loda-lab` executable.

=end

Dir.chdir('../rust_project') do
    `cargo build --release`
end

Dir.chdir('..') do
    `cp rust_project/target/release/loda-lab script/data/loda-lab`
end
