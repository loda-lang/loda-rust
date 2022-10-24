require 'toml'
require 'singleton'
require 'pathname'

class Config
    include Singleton
    
    attr_reader :dot_loda_rust
    attr_reader :analytics_dir
    attr_reader :loda_programs_repository
    attr_reader :loda_rust_executable
    attr_reader :loda_cpp_executable
    attr_reader :oeis_stripped_file
    attr_reader :oeis_names_file
    attr_reader :loda_outlier_programs_repository
    attr_reader :loda_submitted_by
    
    def initialize
        rust_project_default_config = Config.pathname_to_default_config_toml_inside_rust_project
        name_dot_loda_rust = '.loda-rust'
        homedir = ENV['HOME']
        dot_loda_rust = File.join(homedir, name_dot_loda_rust)
        path = File.join(dot_loda_rust, 'config.toml')
        dict_custom = TOML.load_file(path)
        dict_fallback = TOML.load_file(rust_project_default_config)
        dict = dict_fallback.merge(dict_custom)
        
        @dot_loda_rust = dot_loda_rust
        @analytics_dir = File.join(dot_loda_rust, 'analytics')
        @loda_programs_repository = Config.resolve_path(dict, 'loda_programs_repository', homedir)
        @loda_rust_executable = Config.resolve_path(dict, 'loda_rust_executable', homedir)
        @loda_cpp_executable = Config.resolve_path(dict, 'loda_cpp_executable', homedir)
        @oeis_stripped_file = Config.resolve_path(dict, 'oeis_stripped_file', homedir)
        @oeis_names_file = Config.resolve_path(dict, 'oeis_names_file', homedir)
        @loda_submitted_by = dict['loda_submitted_by']
        @loda_outlier_programs_repository = Config.resolve_path(dict, 'loda_outlier_programs_repository', homedir)
    end
    
    def self.pathname_to_default_config_toml_inside_rust_project
        path_from_root_to_default_config_toml = "rust_project/loda-rust-cli/src/config/default_config.toml"
        pathname = pathname_to_loda_rust_dir + path_from_root_to_default_config_toml
        unless pathname.file?
            raise "Unable to find #{pathname.inspect}"
        end
        pathname
    end
    
    def self.pathname_to_loda_rust_dir
        pn = Pathname.getwd
        100.times do
            name = pn.basename.to_s
            if name == 'loda-rust'
                return pn
            end
            pn = pn.parent
            if pn == nil || pn.to_s == "/"
                raise "Unable to find root dir of loda-rust repo"
            end
        end
        raise "Too many attempts. Unable to find loda-rust repo"
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

    def analytics_dir_programs_invalid_file
        File.join(@analytics_dir, 'programs_invalid.csv')
    end

    def analytics_dir_program_rank_file
        File.join(@analytics_dir, 'program_rank.csv')
    end

    def loda_outlier_programs_repository_oeis_divergent
        File.join(@loda_outlier_programs_repository, 'oeis_divergent')
    end
end
