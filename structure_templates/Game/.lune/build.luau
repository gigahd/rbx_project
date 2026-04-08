--!strict
--[[
    Build pipeline. Run with `lune run build`.

    Steps:
        1. Regenerate Blink network code
        2. Install Wally packages
        3. Generate package types for luau-lsp
        4. Generate sourcemap
        5. Build .rbxl via Rojo

    Override output with `lune run build -- --output custom.rbxl`.
]]

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

run("blink", { "Network/Blink/config.Blink" }, "Generating network code")
run("wally", { "install" }, "Installing Wally packages")
run("rojo", { "sourcemap", "default.project.json", "--output", "sourcemap.json" }, "Generating sourcemap")
run("wally-package-types", { "--sourcemap", "sourcemap.json", "Packages/" }, "Generating package types")
run("rojo", { "build", "default.project.json", "--output", OUTPUT }, `Building {OUTPUT}`)

stdio.write(`\n{stdio.color("green")}✓ Build complete: {OUTPUT}{stdio.color("reset")}\n`)
