---
name: preset-share-import
description: Design spec for exporting and importing preset JSON files via native share sheet and file picker
metadata:
  type: project
---

# Preset Share & Import — Design Spec

**Date:** 2026-06-05  
**Status:** Approved

## Overview

Add two AppBar icon buttons — import and export — so users can exchange preset `.json` files with other devices via the native share sheet (Android) or file manager (Linux).

## Packages

Add to `pubspec.yaml`:

```yaml
share_plus: ^10.0.0   # native share sheet with file attachment
file_picker: ^8.0.0   # native file browser filtered to .json
```

No extra native config required for Android or Linux.

## UI

Two new `actions` in `PedalboardScreen`'s `AppBar`:

| Icon | Tooltip | Enabled when |
|------|---------|--------------|
| `Icons.upload_file` | Import preset | Always |
| `Icons.ios_share` | Export preset | At least one preset exists |

The `PresetBar` (below the AppBar) is unchanged.

## Export Flow

1. User taps the export icon.
2. Read active preset: `presetListProvider` + `activePresetIndexProvider`.
3. Write `preset.toJson()` to `<tempDir>/<preset.name>.json` via `path_provider`.
4. Call `SharePlus.instance.shareXFiles([XFile(tempPath)])` → triggers native share sheet.
5. Delete temp file after share completes.

**Error handling:** Button is disabled (grayed out) when no presets exist — no runtime error path needed.

## Import Flow

1. User taps the import icon.
2. Open `FilePicker.platform.pickFiles(type: FileType.custom, allowedExtensions: ['json'])`.
3. If cancelled → no-op.
4. Read file bytes → `jsonDecode` → `Preset.fromJson()`.
5. If parse fails → show `SnackBar("Invalid preset file")`.
6. Check name conflict against `presetListProvider`:
   - **No conflict** → save directly, navigate to it.
   - **Conflict** → show dialog:  
     _"A preset named X already exists."_  
     Actions: **Overwrite** | **Save as copy** (name becomes `"X - imported"`).
7. Call `presetListProvider.notifier.saveCurrentAs(name, pedals)`, reload list, jump to new preset.

## Code Structure

- Logic lives as private methods (`_exportPreset`, `_importPreset`) directly on `PedalboardScreen` (converted to `ConsumerStatefulWidget`) or in a thin helper — no new provider, no new file strictly required.
- `PresetRepository` interface and `FilePresetRepository` are untouched.
- New dependencies on `presetListProvider` and `activePresetIndexProvider` (already used by `PresetBar`).

## Out of Scope

- Registering an Android intent filter to receive `.json` files shared from other apps.
- Bulk export/import of multiple presets at once.
- Any preset format versioning or migration.
