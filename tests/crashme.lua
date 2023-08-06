function crashme(password)
    if string.len(password) ~= 4 then
        return
    end

    if string.sub(password, 1, 1) == "b" then
        if string.sub(password, 2, 2) == "a" then
            if string.sub(password, 3, 3) == "d" then
                if string.sub(password, 4, 4) == "!" then
                    assert(false)
                end
            end
        end
    end
end
