require 'net/http'
require 'uri'
require 'csv'

ENDPOINT_UPLOAD_PROGRAM = "http://api.loda-lang.org/miner/v1/programs"

# This script takes a parameter, a path to a CSV file.
#
# The CSV file looks like this. Each of the rows in the CSV file is an absolute path to a program.
# These programs are to be uploaded to loda-lang.org.
#
# input.csv
# /absolute/path/to/loda/programs/oeis/001/A001281.asm
# /absolute/path/to/loda/programs/oeis/047/A047315.asm
# /absolute/path/to/loda/programs/oeis/314/A314736.asm
# /absolute/path/to/loda/programs/oeis/314/A314848.asm
#
# In order to prepare `input.csv`. If have committed files to my local loda-programs repo, and I want to
# upload these files to loda-lang.org, I do like this.
# git diff-tree --no-commit-id --name-only -r 0b98d576e9c562f80a97e37f9487105a4e8a412a > input.csv
# Next I prefix the rows with their absolute paths.
# Finally I run this script with the input.csv file.

unless ARGV.count == 1
    raise "There must be exactly 1 parameter, the path to a CSV file containing paths to be uploaded"
end
input_csv_path = ARGV[0]
unless File.exist?(input_csv_path)
    raise "No such file #{input_csv_path}, cannot run script"
end

file_paths = []
CSV.foreach(input_csv_path, col_sep: ";") do |row|
    file_paths << row[0]
end

file_paths.each do |file_path|
    unless File.exist?(file_path)
        raise "Expected all files to be valid in the input csv file, however #{file_path} does not exist."
    end
end

def upload_content(file_path)
    content = IO.read(file_path)
    header = {
        'Content-Type': 'application/octet-stream'
    }
    uri = URI.parse(ENDPOINT_UPLOAD_PROGRAM)
    http = Net::HTTP.new(uri.host, uri.port)
    request = Net::HTTP::Post.new(uri.request_uri, header)
    request.body = content

    start = Time.now
    response = http.request(request)
    elapsed = Time.now - start

    if elapsed > 0.2
        puts "Upload of #{file_path} took a long time. elapsed #{elapsed} seconds."
    end
    unless response.code.to_i == 200
        puts "Expected http status code 200, but got #{response.code} from server. file_path: #{file_path}. response: #{response}"
        return
    end
    puts "Uploaded #{file_path}"
end

start = Time.now
file_paths.each do |file_path|
    upload_content(file_path)
end
elapsed = Time.now - start
puts "Uploadning #{file_paths.count} programs, took #{elapsed} seconds."
