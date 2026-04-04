---
paths:
  - "**/*.cs"
  - "**/*.csproj"
  - "**/*.sln"
  - "**/Directory.Build.props"
applies-to: { languages: [csharp] }
---
# C# Hooks

> This file extends [common/hooks.md](../common/hooks.md) with C# specific content.

## PostToolUse Hooks

Configure in `~/.claude/settings.json`:

- **dotnet format**: Auto-format `.cs` files after edit
- **dotnet build**: Verify compilation after editing C# files
- **dotnet test**: Run affected tests after modifications
