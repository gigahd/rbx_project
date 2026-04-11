local process = require("@lune/process")
local stdio = require("@lune/stdio")



local function exec(program: string, programArgs: { string }, label: string)
    stdio.write(`\n{stdio.color("cyan")}▶ {label}{stdio.color("reset")}\n`)
    local result = process.exec(program, programArgs, { stdio = "inherit" })
    if not result.ok then
        stdio.write(`{stdio.color("red")}✗ {label} failed (exit {result.code}){stdio.color("reset")}\n`)
        process.exit(result.code)
    end
end

return exec