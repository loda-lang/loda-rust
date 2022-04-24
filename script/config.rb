require 'toml'
require 'singleton'

class Config
    include Singleton
    
    attr_reader :dot_loda_rust
    attr_reader :analytics_dir
    attr_reader :loda_programs_repository
    attr_reader :loda_cpp_repository
    attr_reader :loda_cpp_executable
    attr_reader :oeis_stripped_file
    attr_reader :oeis_names_file
    attr_reader :loda_rust_mismatches
    attr_reader :loda_submitted_by
    
    def initialize
        name_dot_loda_rust = '.loda-rust'
        dot_loda_rust = File.join(ENV['HOME'], name_dot_loda_rust)
        path = File.join(dot_loda_rust, 'config.toml')
        dict = TOML.load_file(path)
        
        @dot_loda_rust = dot_loda_rust
        @analytics_dir = File.join(dot_loda_rust, 'analytics')
        @loda_programs_repository = dict['loda_programs_repository']
        @loda_cpp_repository = dict['loda_cpp_repository']
        @loda_cpp_executable = dict['loda_cpp_executable']
        @oeis_stripped_file = dict['oeis_stripped_file']
        @oeis_names_file = dict['oeis_names_file']
        @loda_rust_mismatches = dict['loda_rust_mismatches']
        @loda_submitted_by = dict['loda_submitted_by']
    end

    def loda_programs_oeis
        File.join(@loda_programs_repository, 'oeis')
    end
    
    def dot_loda_rust_mine_event
        File.join(@dot_loda_rust, 'mine-event')
    end
    
    def analytics_dir_dont_mine_file
        File.join(@analytics_dir, 'dont_mine.csv')
    end

    def analytics_dir_dependencies_file
        File.join(@analytics_dir, 'dependencies.csv')
    end

    def analytics_dir_program_rank_file
        File.join(@analytics_dir, 'program_rank.csv')
    end
end
