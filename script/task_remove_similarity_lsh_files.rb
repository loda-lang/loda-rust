# Delete all files with the suffix: `similarity_lsh.csv`
Dir.chdir('data/instructions') do
    `find . -type f -name '*similarity_lsh.csv' -delete`
end
