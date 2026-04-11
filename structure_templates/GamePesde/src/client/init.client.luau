local ReplicatedStorage = game:GetService("ReplicatedStorage")

local OnBoot = require(ReplicatedStorage.Packages.OnBoot)

OnBoot.SetDebugInfo(false)

local Shared = ReplicatedStorage:WaitForChild("Shared")
local Services = Shared:WaitForChild("Services")
local UI = Shared:WaitForChild("UI")

local services = OnBoot.LoadChildren(Services)

--Initiate Services
OnBoot.SpawnAll(services, "init", function()
    OnBoot.SpawnAll(services, "start", function()
        print("Client Services Loaded")
    end)
end)

--Initiate UI
OnBoot.SpawnAll(OnBoot.LoadModule(UI))
