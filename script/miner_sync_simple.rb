#!/usr/bin/env ruby

# The "simple" way to "sync"
#
# Purpose
# Intended to be "simple" to use.
# It downloads the latest "loda-programs" repo and 
# replaces the existing "loda-programs" dir.
#
# WARNING: This will destroy any local modifications to "loda-programs".
# Don't use this code if you have LODA programs that you want to keep.
# If you are just mining for LODA programs, then this approach is fine.

require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis

def git_fetch
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git fetch"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def git_latest_commit_id_of_remote_repo
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git ls-remote origin HEAD"
        output = `#{command}`
        output.strip!
        # extract from the string "7a68a9faec4f0dd7c31a70676dcac3fcb942ed75 HEAD"
        # this commit_id "7a68a9faec4f0dd7c31a70676dcac3fcb942ed75"
        if output =~ /\b([0-9a-fA-F]{14,})\b/
            return $1
        end
        puts "error: no commit_id found in command output"
        puts "output from command: #{command}"
        puts output
        raise "no commit_id found in command output"
    end
    raise "could not chdir"
end

def git_current_commit_id_of_local_repo
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git rev-parse HEAD"
        output = `#{command}`
        output.strip!
        # extract commit_id from the output "7a68a9faec4f0dd7c31a70676dcac3fcb942ed75"
        if output =~ /\b([0-9a-fA-F]{14,})\b/
            return $1
        end
        puts "error: no commit_id found in command output"
        puts "output from command: #{command}"
        puts output
        raise "no commit_id found in command output"
    end
    raise "could not chdir"
end

def determine_is_uptodate
    commit_id_remote = git_latest_commit_id_of_remote_repo
    puts "remote commit_id: #{commit_id_remote}"
    commit_id_local = git_current_commit_id_of_local_repo
    puts "local commit_id: #{commit_id_local}"
    is_same = commit_id_remote == commit_id_local
    puts "is_same_commit_id: #{is_same}"
    is_same
end

def determine_has_uncommited_changes
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git status --porcelain"
        output = `#{command}`
        output.strip!
        if output.length == 0
            # no output when there isn't anything changed
            return false
        end
        # outputs 1 or more lines with the files that have been changed
        return true
    end
    raise "could not chdir"
end

# Remove untracked files from the working tree
def git_clean
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git clean -fdx"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def git_reset_hard
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git reset --hard origin/main"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def git_checkout_main_branch
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git checkout main"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def git_pull
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git pull"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def main
    git_fetch
    is_uptodate = determine_is_uptodate
    has_uncommited_changes = determine_has_uncommited_changes
    is_fully_in_sync = is_uptodate && !has_uncommited_changes
    puts "is_fully_in_sync: #{is_fully_in_sync} is_uptodate: #{is_uptodate} is_fully_in_sync: #{is_fully_in_sync}"
    if is_fully_in_sync
        puts "Already up to date. No changes to the official loda-programs repo."
        puts "status: nochange"
        return
    end
    puts "Obtaining the latest snapshot of the official loda-programs repo."
    git_clean
    git_reset_hard
    git_checkout_main_branch
    git_pull
    puts "The local repo is now uptodate."
    puts "status: changed"
end

main
