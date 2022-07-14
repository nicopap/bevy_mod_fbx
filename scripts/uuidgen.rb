#!/usr/bin/env ruby
# Generate derive macro which contains random UUID value.

require 'securerandom'

# Helper class for colored strings
class String
  def red
    "\e[31m#{self}\e[0m"
  end

  def green
	"\e[32m#{self}\e[0m"
  end

  def blue
	"\e[34m#{self}\e[0m"
  end
end

puts 'INFO'.green + ': Generated new derive macro with UUID for you'
puts 'INFO'.green + + ":" + " #[derive(uuid = \"#{SecureRandom.uuid}\")]".blue
