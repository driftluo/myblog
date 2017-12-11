local redis_key = ARGV[1]
local ip = ARGV[2]
local len = redis.call("llen", redis_key);

redis.call("lpush", redis_key, ip)

if len > 100 then
    redis.call("ltrim", redis_key, 0, 100 - 1)
end

return true
