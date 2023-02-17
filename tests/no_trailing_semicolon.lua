a();
function func()
    b();
    c(); -- this is not allowed
    d();
end

--this is allowed ;
--[[
    ;
this is not allowed                 
]]
if a then
    b(); -- this is not allowed
end