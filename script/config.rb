require 'toml'
require 'singleton'

class Config
    include Singleton
    
    attr_reader :loda_programs_repository
    attr_reader :loda_cpp_repository
    attr_reader :loda_cpp_executable
    attr_reader :oeis_names_file
    
    def initialize
        path = File.join(ENV['HOME'], '/.loda-rust/config.toml')
        dict = TOML.load_file(path)
        @loda_programs_repository = dict['loda_programs_repository']
        @loda_cpp_repository = dict['loda_cpp_repository']
        @loda_cpp_executable = dict['loda_cpp_executable']
        @oeis_names_file = dict['oeis_names_file']
    end

    def loda_programs_oeis
        File.join(@loda_programs_repository, 'oeis')
    end
end
