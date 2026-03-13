local ReplicatedStorage = game:GetService("ReplicatedStorage")

local onboot = require(ReplicatedStorage.Packages.onboot)

onboot.SetDebugInfo(true)

local services = onboot.LoadChildren({ script.Services, ReplicatedStorage.Shared.SharedServices })

--Initiate Services
onboot.SpawnAll(services, "init", function()
    onboot.SpawnAll(services, "start", function()
        print("Server Services Loaded")
    end)
end)
