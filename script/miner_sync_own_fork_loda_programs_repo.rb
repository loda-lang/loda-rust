#!/usr/bin/env ruby

require_relative 'config'

LODA_PROGRAMS_OEIS = Config.instance.loda_programs_oeis
REMOTE_NAME = "official_loda-programs"

def git_fetch
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git fetch '#{REMOTE_NAME}'"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

# Usage
# found = git_branch_contains("0007b13acb45e91185219114f7e43ecd7ce1a528")
def git_branch_contains(commit_id)
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git branch --contains #{commit_id}"
        output = `#{command}`
        output.strip!
        if output.length == 0
            return false
        end
        if output =~ /\bmain\b/
            return true
        end
        puts "output from command: #{command}"
        puts output
        raise "Expected either no output, or an output containing 'main'"
    end
    raise "could not chdir"
end

def git_latest_commit_id_of_remote_repo
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git ls-remote '#{REMOTE_NAME}' HEAD"
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

def determine_if_latest_commit_is_present_in_local_repo
    commit_id = git_latest_commit_id_of_remote_repo
    puts "remote repo latest commit_id: #{commit_id}" 
    is_present_in_local_repo = git_branch_contains(commit_id)
    puts "is_present_in_local_repo: #{is_present_in_local_repo}"
    is_present_in_local_repo
end

def git_add
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git add ."
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def has_uncommited_changes
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

def git_commit
    if !has_uncommited_changes
        puts "Nothing to commit"
        return
    end
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git commit -m 'Updated programs'"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def git_merge
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git merge --strategy-option theirs --no-edit 'remotes/#{REMOTE_NAME}/main'"
        output = `#{command}`
        output.strip!
        if output.length > 0
            puts "output from command: #{command}"
            puts output
        end
    end
end

def git_push
    Dir.chdir(LODA_PROGRAMS_OEIS) do
        command = "git push"
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
    is_uptodate = determine_if_latest_commit_is_present_in_local_repo
    if is_uptodate
        puts "Already up to date. No changes to the official loda-programs repo."
        puts "status: nochange"
        return
    end
    puts "Obtaining the latest snapshot of the official loda-programs repo."
    git_add
    git_commit
    git_merge
    git_push
    puts "The local repo is now uptodate."
    puts "status: changed"
end

main
