# Dungeon Crawler Game Jam 2024 Entry
## DungeonRuga

This is my entry for the 2024 Dungeron Crawler Game Jam titled
"DungeonRuga", which is a combination of Dungeon and "Ikarugra".
Ikaruga is a bullet hell top-down shooter from ages long past,
with a very simple concept, where your ship can either be black or
white.  When black, you have black shields and black bullets,
and white shields and white bullets when you're white.  Getting hit
by bullets of the same color do no damage, opposites hurt.

In DungeonRuga, same color magic attacks heal both you and your 
enemies, opposite color attacks hurt.

The game takes place on a 2D grid, rotated 90 degrees.  You can
move up/down/forwards backwards.  Enemies also move on the grid,
however attacks travel on line-of-sight.

When moving, the camera will snap to grid, with the exception of
moving up and down.  While moving up and down, you may move the
camera up and down, so you can see where you are going.  I find
it is disorienting without this behavior.

## The Setting

You have been abducted and dropped in a monster filled alien ant-farm.
You must find the space-ship and escape.  Thankfully you are a powerful
magic user, you may survive.  Who your captors are or if they are watching
may never be known.

The ant farm is hyper-dimensional.  Somehow you can move forward and
backward infinitely, try not to shoot yourself in the back!

## Movement Concept
Moving up and down is done by climbing.  You can "stem" to ascend/descend
tunnels if there are two opposing walls.  You may also stand across
pits the size of one room.  You will fall if there aren't enough
walls to support you.

## Navigation

Press F1 to bring up a map of where you are and rooms visited.
There is a Fog of War.  There are also lamp sprites sprinkled 
around the map to guide your way.  Remember to try moving up and down.

## Implementation details

Tested on Windows and Mac.  I wrote it without an engine in Rust 
version 1.74.  I use macroquad to get a raw pixel buffer, but otherwise
it's an old school raycaster, similar to Doom.

## Running and Building.

### Running

Run the "dungeonjam" executable in the root folder.  The sprites folder and
level.json should all be in the same folder.

You may also build and run from source by installing Rust,
then in root folder type "cargo run --release".

### Running precompiled binaries

I have provided a precompiler binary for mac and Windows. Simply run
them.  If they fail to load, try running from command line in the root 
folder.

### Level Editor

Press F8 to bring up the level editor if you want to cheat.  This is
not documented, look at "level.rs" to figure out the key bindings.