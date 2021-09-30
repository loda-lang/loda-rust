require 'toml'
require 'singleton'

class Config
    include Singleton
    
    attr_reader :dot_loda_rust
    attr_reader :loda_programs_repository
    attr_reader :loda_cpp_repository
    attr_reader :loda_cpp_executable
    attr_reader :oeis_stripped_file
    attr_reader :oeis_names_file
    
    def initialize
        name_dot_loda_rust = '.loda-rust'
        dot_loda_rust = File.join(ENV['HOME'], name_dot_loda_rust)
        path = File.join(dot_loda_rust, 'config.toml')
        dict = TOML.load_file(path)
        
        @dot_loda_rust = dot_loda_rust
        @loda_programs_repository = dict['loda_programs_repository']
        @loda_cpp_repository = dict['loda_cpp_repository']
        @loda_cpp_executable = dict['loda_cpp_executable']
        @oeis_stripped_file = dict['oeis_stripped_file']
        @oeis_names_file = dict['oeis_names_file']
    end

    def loda_programs_oeis
        File.join(@loda_programs_repository, 'oeis')
    end
    
    def dot_loda_rust_mine_event
        File.join(@dot_loda_rust, 'mine-event')
    end
end
