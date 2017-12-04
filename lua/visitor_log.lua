local redis_key = ARGV[1]
local ip = ARGV[2]
local len = redis.call("llen", redis_key);

redis.call("rpush", redis_key, ip)

if len > 100 then
    redis.call("ltrim", redis_key, len - 99, len)
end

return true
