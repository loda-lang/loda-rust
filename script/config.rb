require 'toml-rb'
require 'singleton'

class Config
    include Singleton
    
    attr_reader :loda_programs_repository
    attr_reader :loda_cpp_repository
    
    def initialize
        path = File.join(ENV['HOME'], '/.loda-rust/config.toml')
        dict = TomlRB.load_file(path)
        @loda_programs_repository = dict['loda_programs_repository']
        @loda_cpp_repository = dict['loda_cpp_repository']
    end

    def loda_program_rootdir
        File.join(@loda_programs_repository, 'oeis')
    end
end
