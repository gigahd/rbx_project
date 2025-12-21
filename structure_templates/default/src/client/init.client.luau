local ReplicatedStorage = game:GetService("ReplicatedStorage")

local serviceList = {}
function addService(serviceModule: Instance)
    if serviceModule:IsA("ModuleScript") then
        local service = require(serviceModule)
        if service.init then
            service:init()
        end
        table.insert(serviceList, service)
    end
end

for _, serviceModule in ReplicatedStorage.Shared.SharedServices:GetChildren() do
    addService(serviceModule)
end

for _, serviceModule in ReplicatedStorage.Shared.Services:GetChildren() do
    addService(serviceModule)
end

for _, service in serviceList do
    if service.start then
        service:start()
    end
end
