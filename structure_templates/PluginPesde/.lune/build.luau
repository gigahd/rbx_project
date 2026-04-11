--!strict
--[[
    Build pipeline. Run with `lune run build`.

    Steps:
        1. Install pesde packages
        2. Generate sourcemap
        3. Build .rbxm via Rojo

    Override output with `lune run build -- --output custom.rbxm`.
]]

local process = require("@lune/process")
local stdio = require("@lune/stdio")
local run = require("./Util/exec")

local OUTPUT = "build/plugin.rbxm"

local args = process.args
for i, arg in args do
    if arg == "--output" and args[i + 1] then
        OUTPUT = args[i + 1]
    end
end

run("pesde", { "install" }, "Installing pesde packages")
run("rojo", { "sourcemap", "default.project.json", "--output", "sourcemap.json" }, "Generating sourcemap")
run("rojo", { "build", "default.project.json", "--output", OUTPUT }, `Building {OUTPUT}`)

stdio.write(`\n{stdio.color("green")}✓ Build complete: {OUTPUT}{stdio.color("reset")}\n`)
