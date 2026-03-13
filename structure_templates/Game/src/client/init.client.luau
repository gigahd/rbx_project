local ReplicatedStorage = game:GetService("ReplicatedStorage")

local onboot = require(ReplicatedStorage.Packages.onboot)

onboot.SetDebugInfo(true)

local services = onboot.LoadChildren({ ReplicatedStorage.Shared.Services, ReplicatedStorage.Shared.SharedServices })

--Initiate Services
onboot.SpawnAll(services, "init", function()
    onboot.SpawnAll(services, "start", function()
        print("Client Services Loaded")
    end)
end)

--Initiate UI
onboot.SpawnAll(onboot.LoadModule(ReplicatedStorage.UI))