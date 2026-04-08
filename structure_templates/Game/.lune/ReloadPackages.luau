local process = require("@lune/process")
local stdio = require("@lune/stdio")

local OUTPUT = "build/game.rbxl"

local args = process.args
for i, arg in args do
    if arg == "--output" and args[i + 1] then
        OUTPUT = args[i + 1]
    end
end

local function run(program: string, programArgs: { string }, label: string)
    stdio.write(`\n{stdio.color("cyan")}▶ {label}{stdio.color("reset")}\n`)
    local result = process.spawn(program, programArgs, { stdio = "inherit" })
    if not result.ok then
        stdio.write(`{stdio.color("red")}✗ {label} failed (exit {result.code}){stdio.color("reset")}\n`)
        process.exit(result.code)
    end
end

run("wally", { "install" }, "Installing Wally packages")
run("rojo", { "sourcemap", "default.project.json", "--output", "sourcemap.json" }, "Generating sourcemap")
run("wally-package-types", { "--sourcemap", "sourcemap.json", "Packages/" }, "Generating package types")
run("rojo", { "serve" }, "Starting Rojo server")