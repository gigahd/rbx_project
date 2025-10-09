local ServerScriptService = game:GetService("ServerScriptService")

local serviceList = {}
for _, serviceModule in ServerScriptService.Services:GetChildren() do
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
