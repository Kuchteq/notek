-- Myers diff algorithm (character-level)
-- Returns a list of edits: { op="insert"/"delete", pos=number, char=string }
local function diff(e, f, i, j)
    i = i or 0
    j = j or 0
    local N, M = #e, #f
    local L = N + M
    local Z = 2 * math.min(N, M) + 2
    
    if N > 0 and M > 0 then
        local w = N - M
        local g, p = {}, {}
        -- Initialize arrays with 0
        for idx = 0, Z - 1 do g[idx] = 0; p[idx] = 0 end

        for h = 0, math.floor(L / 2 + (L % 2 ~= 0 and 1 or 0)) do
            for r = 0, 1 do
                local c, d, o, m
                if r == 0 then
                    c, d, o, m = g, p, 1, 1
                else
                    c, d, o, m = p, g, 0, -1
                end

                local start_k = -(h - 2 * math.max(0, h - M))
                local end_k = h - 2 * math.max(0, h - N)
                
                for k = start_k, end_k, 2 do
                    local a
                    if k == -h or (k ~= h and c[(k - 1) % Z] < c[(k + 1) % Z]) then
                        a = c[(k + 1) % Z]
                    else
                        a = c[(k - 1) % Z] + 1
                    end
                    
                    local b = a - k
                    local s, t = a, b
                    
                    -- String indexing in Lua: string.sub(str, pos, pos)
                    -- We use (1-o)*N + m*a + o for 1-based adjustment
                    while a < N and b < M do
                        local char_e = string.sub(e, (1 - o) * N + m * a + (o == 1 and 1 or 0), (1 - o) * N + m * a + (o == 1 and 1 or 0))
                        local char_f = string.sub(f, (1 - o) * M + m * b + (o == 1 and 1 or 0), (1 - o) * M + m * b + (o == 1 and 1 or 0))
                        if char_e ~= char_f then break end
                        a, b = a + 1, b + 1
                    end
                    
                    c[k % Z] = a
                    local z = -(k - w)
                    
                    if L % 2 == o and z >= -(h - o) and z <= (h - o) and (c[k % Z] + d[z % Z] >= N) then
                        local D, x, y, u, v
                        if o == 1 then
                            D, x, y, u, v = 2 * h - 1, s, t, a, b
                        else
                            D, x, y, u, v = 2 * h, N - a, M - b, N - s, M - t
                        end

                        if D > 1 or (x ~= u and y ~= v) then
                            local part1 = diff(string.sub(e, 1, x), string.sub(f, 1, y), i, j)
                            local part2 = diff(string.sub(e, u + 1, N), string.sub(f, v + 1, M), i + u, j + v)
                            -- Table concatenation
                            for _, val in ipairs(part2) do table.insert(part1, val) end
                            return part1
                        elseif M > N then
                            return diff("", string.sub(f, N + 1, M), i + N, j + N)
                        elseif M < N then
                            return diff(string.sub(e, M + 1, N), "", i + M, j + M)
                        else
                            return {}
                        end
                    end
                end
            end
        end
    elseif N > 0 then
        local res = {}
        for n = 0, N - 1 do
            table.insert(res, {operation = "delete", position_old = i + n})
        end
        return res
    else
        local res = {}
        for n = 0, M - 1 do
            table.insert(res, {operation = "insert", position_old = i, position_new = j + n})
        end
        return res
    end
end

