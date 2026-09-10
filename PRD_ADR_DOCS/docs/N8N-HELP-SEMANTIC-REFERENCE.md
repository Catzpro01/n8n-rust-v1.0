# n8n 2.39.0 CLI Help Semantic Reference
## Extracted from Source Code (Not Byte-Perfect)

**Source:** /opt/agent-workspace/upstream/n8n-2.39.0/packages/cli/src/commands/
**Extraction Method:** Source code analysis (not runtime --help output)
**Paritas Level:** L3 Semantic (per CLI-PARITY-SPEC)
**Date:** 2026-09-09
**Extractor:** agent2

---

## 6 Commands Wajib Paritas (matt R67)

### 1. Root --help
**Description:** Auto-generated oleh oclif framework dari command list
**Commands listed:** start, execute, import:workflow, list:workflow, export:workflow, + others
**Note:** Root help is auto-generated, not defined in single source file

---

### 2. start
**Name:** start
**Description:** Starts n8n. Makes Web-UI available and starts active workflows
**Examples:**
- `start` (no args)
- `start -o` (open browser)

**Flags:**
- `--open` (alias: `-o`) - boolean, optional - opens the UI automatically in browser

**Source:** packages/cli/src/commands/start.ts

---

### 3. import:workflow
**Name:** import:workflow
**Description:** Import workflows
**Examples:**
- `--input=file.json`
- `--separate --input=backups/latest/`
- `--input=file.json --userId=1d64c3d2-85fe-4a83-a649-e446b07b3aae`
- `--input=file.json --projectId=Ox8O54VQrmBrb4qL`
- `--separate --input=backups/latest/ --userId=1d64c3d2-85fe-4a83-a649-e446b07b3aae`
- `--input=file.json --activeState=fromJson`

**Flags:**
- `--input` (alias: `-i`) - string, optional - Input file name or directory if --separate is used
- `--separate` - boolean, default false - Imports *.json files from directory provided by --input
- `--userId` - string, optional - The ID of the user to assign the imported workflows to
- `--projectId` - string, optional - The ID of the project to assign the imported workflows to
- `--activeState` - enum ['false', 'fromJson'], default 'false' - Whether to respect the JSON active field. "false" (default) deactivates all imported workflows. "fromJson" activates/deactivates each workflow based on its JSON active field.

**Source:** packages/cli/src/commands/import/workflow.ts

---

### 4. list:workflow
**Name:** list:workflow
**Description:** List workflows
**Examples:**
- `list:workflow` (no args)
- `list:workflow --active=true --onlyId`
- `list:workflow --active=false`

**Flags:**
- `--active` - string, optional - Filters workflows by active status. Can be true or false
- `--onlyId` - boolean, default false - Outputs workflow IDs only, one per line.

**Source:** packages/cli/src/commands/list/workflow.ts

---

### 5. execute
**Name:** execute
**Description:** Executes a given workflow
**Examples:**
- `execute --id=5`

**Flags:**
- `--id` - string, optional - id of the workflow to execute
- `--rawOutput` - boolean, optional - Outputs only JSON data, with no other text
- `--file` - string, optional, DEPRECATED - DEPRECATED: Please use --id instead

**Source:** packages/cli/src/commands/execute.ts

---

### 6. export:workflow
**Name:** export:workflow
**Description:** Export workflows
**Examples:**
- `export:workflow --all`
- `export:workflow --projectId=Ox8O54VQrmBrb4qL`
- `export:workflow --id=5 --output=file.json`
- `export:workflow --id=5 --version=abc-123-def`
- `export:workflow --id=5 --published`
- `export:workflow --all --published --output=backups/latest/`
- `export:workflow --all --output=backups/latest/`
- `export:workflow --backup --output=backups/latest/`

**Flags:**
- `--all` - boolean, optional - Export all workflows
- `--backup` - boolean, optional - Sets --all --pretty --separate for simple backups. Only --output has to be set additionally.
- `--id` - string, optional - The ID of the workflow to export
- `--projectId` - string, optional - Export all workflows in the specified project
- `--output` (alias: `-o`) - string, optional - Output file name or directory if using separate files
- `--pretty` - boolean, optional - Format the output in an easier to read fashion
- `--separate` - boolean, optional - Exports one file per workflow (useful for versioning). Must inform a directory via --output.
- `--version` - string, optional - The version ID to export
- `--published` - boolean, optional - Export the published/active version

**Source:** packages/cli/src/commands/export/workflow.ts

---

## Paritas Requirements (L3 Semantic)

Per CLI-PARITY-SPEC L3:
- ✅ **Description identik** - Text description must match exactly
- ✅ **Flags identik** - Same flags with same types and descriptions
- ✅ **Urutan identik** - Flags listed in same order
- ✅ **Layout boleh beda** - Formatting/alignment can differ

**NOT required:**
- ❌ Byte-perfect output (spacing, alignment)
- ❌ Exact whitespace matching
- ❌ Terminal width adaptation

---

## Verification Commands

Untuk verify paritas semantic:
```bash
# Check description matches
grep "description:" <source-file>

# Check flags match
grep -A5 "flagsSchema = z.object" <source-file>

# Check examples match
grep -A10 "examples:" <source-file>
```

---

## Source Files Referenced

1. packages/cli/src/commands/start.ts
2. packages/cli/src/commands/import/workflow.ts
3. packages/cli/src/commands/list/workflow.ts
4. packages/cli/src/commands/execute.ts
5. packages/cli/src/commands/export/workflow.ts

---

## Notes

- n8n 2.39.0 upstream source (not built/compiled)
- Extracted via source code analysis, not runtime execution
- Semantic paritas level L3 (description + flags + order)
- Root --help is auto-generated by oclif framework
- All flag descriptions extracted verbatim from source

---

**Status:** COMPLETE
**Ready for:** TB-04 implementation reference
