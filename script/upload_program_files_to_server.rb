# Batch uploading of many program files to loda-lang.org.

require 'net/http'
require 'uri'

ENDPOINT_UPLOAD_PROGRAM = "http://api.loda-lang.org/miner/v1/programs"
FOOTER_WITH_MINER_PROFILE = "\n; Miner Profile: loda-rust\n"

def upload_program_file_to_server(file_path)
    content = IO.read(file_path)

    # Append footer to program for metrics
    content.strip!
    content += FOOTER_WITH_MINER_PROFILE

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
    status_code = response.code.to_i
    upload_success = status_code == 200 || status_code == 201
    unless upload_success
        puts "Expected http status code 200 or 201, but got #{response.code} from server. file_path: #{file_path}. response: #{response}"
        return
    end
    puts "Uploaded #{file_path}, status: #{status_code}"
end

def upload_program_files_to_server(file_paths)
    file_paths.each do |file_path|
        unless File.exist?(file_path)
            raise "Expected all file_paths to be valid, however #{file_path} does not exist."
        end
    end

    start = Time.now
    file_paths.each do |file_path|
        upload_program_file_to_server(file_path)
    end
    elapsed = Time.now - start
    puts "Uploadning #{file_paths.count} programs, took #{elapsed} seconds."
end
