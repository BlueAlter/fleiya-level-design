# Enums

An **enum** is a fixed list of named values. Use one when a field should accept
only a few known options instead of any text or number. For example, an
`ItemRarity` enum might contain `Common`, `Rare`, and `Epic`.

Each value carries:

- an **id** such as `Common`, used in files and references,
- a **display name** shown to the player or editor user,
- a **badge color** used in the editor UI,
- an optional **tile icon** taken from a tileset.

Enums keep their values in a fixed order, so reordering values is safe while
renaming ids is a structural change.

## Create an enum

1. Press **+ New Enum** in the enum list on the left.
2. Set the **Identifier** to the name other files will use, for example
   `ItemRarity`. It must be unique in the project.
3. Pick a **Tileset** if the values should show icons. This step is optional;
   enums without a tileset still work and simple values need no graphics.
4. Press **+ Add Value** for each entry, then set its **id**, **display name**,
   **badge color**, and **icon tile** (only when a tileset is selected).
5. Drag values to change their order, or use the sort controls.

## Tags

Tags are labels that only **categorize** enums. They help you group and filter
the enum list; they never change enum data, values, or how a field behaves.
Assign a tag with the tag button on the enum card, type to search the existing
tags, and remove a tag with its small cross.

## Where enums are used

- **Entity fields**: an enum field stores one value id and shows a picker with
  the enum's display names and colors.
- **Tile marking** (planned): tilesets will let you mark tiles with an enum, so
  a tile can describe itself as `Grass`, `Water`, or `Road`.

## Safety

- Renaming a value id updates every reference, and the editor warns before an
  id change that would break external files.
- Deleting a value is blocked while something references it; the confirmation
  dialog lists the affected entities and fields.
- Deleting an enum is blocked while any field still uses it.
- Every edit is a command on the undo stack, so a mistaken rename, reorder,
  or delete can be reverted with **Ctrl+Z**.