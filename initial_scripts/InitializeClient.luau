local ReplicatedStorage = game:GetService("ReplicatedStorage")

local serviceList = {}
for _, serviceModule in ReplicatedStorage.Services:GetChildren() do
	if serviceModule:IsA("ModuleScript") then
		local service = require(serviceModule)
		if service.init then
			service:init()
		end
		table.insert(serviceList, service)
	end
end

for _, service in serviceList do
	if service.start then
		service:start()
	end
end
