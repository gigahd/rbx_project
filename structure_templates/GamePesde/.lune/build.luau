--!strict
--[[
    Build pipeline. Run with `lune run build`.

    Steps:
        1. Regenerate Blink network code
        2. Install pesde packages
        3. Generate sourcemap
        4. Build .rbxl via Rojo

    Override output with `lune run build -- --output custom.rbxl`.
]]

local process = require("@lune/process")
local stdio = require("@lune/stdio")
local run = require("./Util/exec")

local OUTPUT = "build/game.rbxl"

local args = process.args
for i, arg in args do
    if arg == "--output" and args[i + 1] then
        OUTPUT = args[i + 1]
    end
end

run("blink", { "Network/Blink/config.Blink" }, "Generating network code")
run("pesde", { "install" }, "Installing pesde packages")
run("rojo", { "sourcemap", "default.project.json", "--output", "sourcemap.json" }, "Generating sourcemap")
run("rojo", { "build", "default.project.json", "--output", OUTPUT }, `Building {OUTPUT}`)

stdio.write(`\n{stdio.color("green")}✓ Build complete: {OUTPUT}{stdio.color("reset")}\n`)
