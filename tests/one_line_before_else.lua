function f(a)
    if a then
    else -- should no error here
        f(not a)
    end
end

if a then

elseif k then -- should no error here

else -- should no error here

end

if a then
    oh()
elseif k then -- should report here
    no()
else -- should report here
end