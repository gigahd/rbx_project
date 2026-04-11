local ReplicatedStorage = game:GetService("ReplicatedStorage")

local OnBoot = require(ReplicatedStorage.Packages.OnBoot)

OnBoot.SetDebugInfo(true)

local services = OnBoot.LoadChildren(script.Services)

--Initiate Services
OnBoot.SpawnAll(services, "init", function()
    OnBoot.SpawnAll(services, "start", function()
        print("Server Services Loaded")
    end)
end)
