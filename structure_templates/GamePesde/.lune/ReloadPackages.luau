local run = require("./Util/exec")

run("pesde", { "install" }, "Installing pesde packages")
run("rojo", { "sourcemap", "default.project.json", "--output", "sourcemap.json" }, "Generating sourcemap")
run("rojo", { "serve" }, "Starting Rojo server")
