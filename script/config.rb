require 'toml'
require 'singleton'

DEFAULT_CONFIG = <<-TOML
# Absolute path to the "loda-programs" repository dir.
loda_programs_repository = "$HOME/loda/programs"

# Absolute path to the "loda-cpp" repository dir.
loda_cpp_repository = "$HOME/git/loda-cpp"

# Absolute path to the "loda" executable file.
loda_cpp_executable = "$HOME/loda/bin/loda"

# Absolute path to the "loda-rust" repository dir.
loda_rust_repository = "$HOME/git/loda-rust"

# Absolute path to the unzipped OEIS stripped file.
oeis_stripped_file = "$HOME/loda/oeis/stripped"

# Absolute path to the unzipped OEIS names file.
oeis_names_file = "$HOME/loda/oeis/names"

# Who to be credited when discovering new programs.
loda_submitted_by = "John Doe"

# When mining with metrics enabled, this is the port that the metrics can be accessed.
miner_metrics_listen_port = 8090

# What loda programs are similar to each other.
loda_identify_similar_programs_repository = "$HOME/git/loda-identify-similar-programs"

# Patterns that are frequently used in loda programs.
loda_patterns_repository = "$HOME/git/loda-patterns"

# Absolute path to the "loda-outlier-programs" repository dir.
loda_outlier_programs_repository = "$HOME/git/loda-outlier-programs"
TOML

class Config
    include Singleton
    
    attr_reader :dot_loda_rust
    attr_reader :analytics_dir
    attr_reader :loda_programs_repository
    attr_reader :loda_cpp_repository
    attr_reader :loda_cpp_executable
    attr_reader :oeis_stripped_file
    attr_reader :oeis_names_file
    attr_reader :loda_outlier_programs_repository
    attr_reader :loda_submitted_by
    
    def initialize
        name_dot_loda_rust = '.loda-rust'
        homedir = ENV['HOME']
        dot_loda_rust = File.join(homedir, name_dot_loda_rust)
        path = File.join(dot_loda_rust, 'config.toml')
        dict_custom = TOML.load_file(path)
        dict_fallback = TOML.load(DEFAULT_CONFIG)
        dict = dict_fallback.merge(dict_custom)
        
        @dot_loda_rust = dot_loda_rust
        @analytics_dir = File.join(dot_loda_rust, 'analytics')
        @loda_programs_repository = Config.resolve_path(dict, 'loda_programs_repository', homedir)
        @loda_cpp_repository = Config.resolve_path(dict, 'loda_cpp_repository', homedir)
        @loda_cpp_executable = Config.resolve_path(dict, 'loda_cpp_executable', homedir)
        @oeis_stripped_file = Config.resolve_path(dict, 'oeis_stripped_file', homedir)
        @oeis_names_file = Config.resolve_path(dict, 'oeis_names_file', homedir)
        @loda_submitted_by = dict['loda_submitted_by']
        @loda_outlier_programs_repository = Config.resolve_path(dict, 'loda_outlier_programs_repository', homedir)
    end
    
    def self.resolve_path(dict, key, homedir)
        path = dict[key]
        raise "config file has no path for key #{key.inspect}" if path == nil
        path2 = path.gsub(/^[$]HOME\//, '')
        if path2.length < path.length
            return File.join(homedir, path2)
        end
        return path
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

    def loda_outlier_programs_repository_oeis_divergent
        File.join(@loda_outlier_programs_repository, 'oeis_divergent')
    end
end
