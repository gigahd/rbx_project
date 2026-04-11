local run = require("./Util/exec")

run("wally", { "install" }, "Installing Wally packages")
run("pesde", { "install" }, "Installing pesde packages")
run("rojo", { "sourcemap", "default.project.json", "--output", "sourcemap.json" }, "Generating sourcemap")
run("wally-package-types", { "--sourcemap", "sourcemap.json", "Packages/" }, "Generating package types")
run("rojo", { "serve" }, "Starting Rojo server")
