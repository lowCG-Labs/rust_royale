The Global Architecture (The 4 Layers of the Game)
Your game will be built in four distinct layers, stacked like a cake.

Layer 1: The Static Data (The "Spreadsheet")

What it is: This is where your confusion gets solved! We will have a simple JSON or CSV file containing the raw Level 11 stats for every entity.

Examples: Princess_Tower_Health: 3052, King_Tower_Health: 4824, Knight_HP: 1674, Knight_Damage: 202.

Why it's Layer 1: The engine reads this file when it boots up. If you want to tweak a stat, you just change the text file; you don't rewrite the Rust code.

Layer 2: The Core Engine (The "Brain")

What it is: The Bevy ECS, the 18x32 Matrix, the A* Pathfinding, the spatial hashing, and the fixed-point math we just documented.

How it uses Layer 1: When you drop a Knight, the engine looks at Layer 1, says "Ah, Level 11 Knight has 1674 HP," and attaches that number to the new entity.

Layer 3: The Multiplayer Sync (The "Network")

What it is: The ggrs rollback crate and the Room ID matchmaker.

How it works: It sits on top of the engine. It doesn't care about Knight health. It only cares about intercepting Player 1 and Player 2's mouse clicks and feeding them into Layer 2 at the exact same frame.

Layer 4: The UI & Presentation (The "Skin")

What it is: The graphics, the dragging animations, the Elixir bar, and the main menu screen.


The Development Roadmap (How we actually build it)
To build this without losing our minds, we cannot build all four layers at once. We build it in Milestones.

Milestone 1: The Offline Sandbox (Data + Environment)
Step A: We create the JSON file holding the Level 11 stats for the Towers and 2 basic cards (e.g., Knight and Musketeer).

Step B: We build the 18x32 Grid in Rust.

Step C: We write a script that reads the JSON file and spawns the Towers onto the grid with their correct 3052 and 4824 health pools.

Goal: You open the app, you see a grid, and you see towers that actually possess health data.

Milestone 2: The Combat Loop (Logic)
Step A: We build the Spawner so clicking the mouse drops a Knight.

Step B: We write the A* pathfinding so the Knight walks to the tower.

Step C: We write the Combat System. The Knight hits the tower, and the tower's health drops from 3052 to 2850. When it hits 0, the tower disappears.

Goal: You can play a primitive, offline game against a dummy opponent on a single screen.

Milestone 3: The Multiplayer (Networking)
Step A: We spin up a tiny web server to handle the "Room ID" handshake.

Step B: We integrate ggrs. Instead of one mouse controlling the game, the engine waits for inputs from both PC A and PC B.

Goal: You and your friend can connect from different houses and drop units on the same synchronized grid.

Milestone 4: The Game Rules (Polish)
Step A: We add the Match Clock, the 2x Elixir rules, and the Deck rotation queue we documented earlier.

Step B: UI polish (health bars, timers).

Goal: The fully playable game.