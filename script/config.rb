require 'toml-rb'
require 'singleton'

class Config
    include Singleton
    
    attr_reader :loda_program_rootdir
    attr_reader :loda_cpp_repository
    
    def initialize
        path = File.join(ENV['HOME'], '/.loda-lab/config.toml')
        dict = TomlRB.load_file(path)
        @loda_program_rootdir = dict['loda_program_rootdir']
        @loda_cpp_repository = dict['loda_cpp_repository']
    end
end
