# 0007. Map images only from the user's game files

- Status: Accepted
- Date: 2026-11-14

## Context
JASE shows map images from two sources: the levelshots inside the game's own
archives, and image.gametracker.com. The second depends on a third party,
sends it one request per map, and its images are not ours to ship
([ADR 0004](0004-no-proprietary-ase-files.md)).

## Decision
Map images come only from the user's installed games: `levelshots/<map>`
in the `.pk3` files of the folder named by the game's `levelshots` field in
`games.json` (Quake 3 engine games), with the server's mod folder first.
Nothing is downloaded and no image ships with OpenASE.

## Consequences
- Works only for installed games with a program location set.
- Call of Duty (IWI load screens) and Unreal packages need their own readers; not done.
- New dependencies: `zip` (deflate via `flate2`) and `image` (JPEG, TGA, PNG).
- A download source can be added later behind an opt-in setting, with a new ADR.
