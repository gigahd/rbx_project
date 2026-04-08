# {{project_name}}

Professional Roblox project using Luau + Rojo, built around the
**onboot** service lifecycle and the **red-blox** async stack.

## Stack

| Layer              | Tool                                  |
| ------------------ | ------------------------------------- |
| Toolchain          | Rokit                                 |
| Sync               | Rojo                                  |
| Packages           | Wally                                 |
| Lint / format      | Selene + StyLua                       |
| Language server    | luau-lsp                              |
| Service lifecycle  | onboot                                |
| State              | Charm (+ CharmSync for replication)   |
| UI                 | Vide                                  |
| Event networking   | Blink (IDL codegen, Future-based)     |
| Async              | red-blox/future                       |
| Cleanup            | Trove                                 |
| Runtime validation | osyrisrblx/t                          |
| Player data        | ProfileStore                          |
| Signals            | sleitnick/signal                      |
| UI previews        | UI Labs                               |
| Build automation   | Lune                                  |
| Testing            | Jest-Lua                              |
| CI/CD              | GitHub Actions + rbxcloud             |
| Docs               | Moonwave                              |

## One-time setup

```bash
rokit install          # installs every tool in rbx_project.toml
wally install          # fetches packages
blink Network/Blink/config.Blink   # generate network code
rojo sourcemap default.project.json --output sourcemap.json
wally-package-types --sourcemap sourcemap.json Packages/
```

Then install the VSCode extensions recommended in `.vscode/extensions.json`
and open the folder — luau-lsp will pick everything up automatically.

## Daily workflow

```bash
# Terminal 1 — live sync to Studio
rojo serve

# Terminal 2 — regenerate net code when config.Blink changes
blink Network/Blink/config.Blink

# Anytime
lune run analyze    # lint + format check + sourcemap
lune run build      # full build pipeline → build/game.rbxl
```

In Studio, install the Rojo plugin and click **Connect**.

## Architecture

### Service lifecycle (onboot)

Every service exports a table with optional `init` and `start` functions.
onboot calls all `init`s first, then all `start`s, so services can safely
reference each other during `start` without caring about load order.

```lua
local MyService = {}
MyService._trove = Trove.new()

function MyService.init()
    -- Create instances, initialize state. Don't connect events yet.
end

function MyService.start()
    -- Wire up event handlers, begin operation.
    MyService._trove:Connect(SomeEvent, handler)
end

function MyService.destroy()
    MyService._trove:Destroy()
end

return MyService
```

Service locations:

- `src/server/Services/` — server-only, loaded from `ServerScriptService.Server.Services`
- `src/shared/Services/` — client-only, loaded from `ReplicatedStorage.Shared.Services`
- `src/shared/SharedServices/` — runs on both sides

### State vs events

The template uses a **clean split** between state replication and event networking:

- **State → Charm + CharmSync.** Anything the client needs to *read*
  (money, health, inventory) lives in a Charm atom in `src/shared/Atoms.luau`.
  The server mutates atoms; `ServerSync` replicates them via `CharmSync`.
  No `MoneyChanged` events needed.

- **Events → Blink.** Things that *happened* (purchase requested, damage
  dealt) are defined in `Network/Blink/config.Blink` as IDL events. Blink
  generates typed, buffer-serialized Luau code. SingleAsync calls return
  Futures directly because `FutureLibrary` is configured.

### Strict Luau everywhere

`.luaurc` sets `"languageMode": "strict"` globally. All modules should
start with `--!strict`. This is the highest-value practice for a solo dev
because the type checker catches bugs at edit time when you don't have
a QA tester.

`t` runtime validation complements strict mode — use it at trust
boundaries (DataStore loads, HTTP responses, pre-Blink RemoteEvent
payloads) where the type system can't help you.

## Directory layout

Game/
├── Assets/
│   ├── Shared/             → ReplicatedStorage.Assets
│   ├── Server/             → ServerStorage.Assets
├── Network/
│   ├── Blink/config.Blink  IDL source
│   ├── Client/             generated (gitignored)
│   └── Server/             generated (gitignored)
├── src/
│   ├── client/
│   │   └── init.client.luau   onboot entry
│   ├── server/
│   │   ├── init.server.luau   onboot entry
│   │   └── Services/          server-only services
│   └── shared/
│       ├── Services/          client services (Shared so they replicate)
│       └── UI/
│           └── init.luau          mount entry
├── lune/
│   ├── build.luau             build pipeline
│   └── analyze.luau           lint/format/check runner
├── .github/workflows/ci.yml
├── default.project.json
├── rbx_project.toml           scaffolder config
├── selene.toml
├── stylua.toml
└── .luaurc

## CI/CD

`.github/workflows/ci.yml` runs on every push:

1. Installs Rokit and all pinned tools
2. Installs Wally packages + generates types
3. Generates Blink network code
4. Runs Selene and StyLua
5. Builds `game.rbxl` and uploads as an artifact
6. On `main`, publishes via `rbxcloud`

Set these in your repo settings:

- **Secret:** `RBXCLOUD_API_KEY` — from the Creator Dashboard, scoped to
  Place Publishing only
- **Variables:** `PLACE_ID`, `UNIVERSE_ID`

## What to customize first

1. Update `default.project.json` `name` field (scaffolder handles `{{project_name}}`)
2. Replace the ProfileStore stub in `PlayerDataService` with real session logic
3. Define your actual events in `Network/Blink/config.Blink`
4. Add your atoms to `Atoms.luau` and register them in both `ServerSync` and `ClientSync`

## Useful commands

```bash
rojo serve                              # live sync to Studio
rojo build -o game.rbxl                 # build place file
wally install                           # install packages
blink Network/Blink/config.Blink        # regenerate network code
selene src/                             # lint
stylua src/                             # format
stylua --check src/                     # format check (CI)
lune run build                          # full build pipeline
lune run analyze                        # all checks
```
