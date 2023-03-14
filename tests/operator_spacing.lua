local a = 1; -- ok
function sub(b, c)
    return b -c -- not ok
end

local c = -1 -- ok
local d =
    1 + sum(1, 2) + sum(2, 3, 4) + sum(3, 4, 5, 6) -- ok

local e = -(1 + 2) -- ok
local f = sum(1, 2) + (-1) -- ok
local g = test({-1}) -- ok
local h = -test({-1}) -- ok
local i = a -b -- not ok
local j = a - b -- ok
