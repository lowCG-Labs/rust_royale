# Physics Engine: Technical Debt & Scaling Notes

As of Phase 2 (Local Avoidance), the engine uses a basic Boids separation algorithm (`troop_collision_system`) to prevent units from overlapping. While stable for small skirmishes, the following optimizations are required before scaling to 50+ units.

### 1. Spatial Optimization ($O(N^2)$ Scaling)
* **The Issue:** We are currently using `iter_combinations_mut()` to compare every single troop's footprint against every other troop. This $O(N^2)$ math will drop the frame rate when unit counts get high.
* **The Solution:** Slated for **Phase 6**. We must implement a Spatial Hash Map (Global Spacing grid). Units should only calculate collision physics against entities in their immediate or adjacent grid buckets.

### 2. High-Density "Boiling" (The Magic 0.5 Force)
* **The Issue:** The separation force multiplier is currently hardcoded to `0.5`. If 10+ units are spawned on a single tile, or crammed into a bridge chokepoint, a `0.5` push might not resolve the overlap in a single frame, causing the cluster to visually "boil" or jitter.
* **The Solution:** Slated for **Phase 3B (Bridge Pathfinding)**. If boiling occurs at the chokepoints, we will either dynamically increase the push force based on overlap severity, or run the collision loop 2-3 times per `FixedUpdate` step for stability.

### 3. Team-Based Friction (Hard vs. Soft Blocking)
* **The Issue:** Right now, Blue troops push Blue troops with the exact same force they push Red troops. 
* **The Solution:** Slated for **Combat Polish**. In real Clash Royale, friendly units "slide" around each other easily, while enemy units act as hard physical walls. We need to bring `&Team` into the collision query. If `team_a != team_b`, the separation force should act as an immovable wall (effectively infinite mass) rather than a shared push.

### 4. Deployment Displacement (Feature, Not a Bug)
* **The Mechanics:** The `troop_collision_system` intentionally does *not* filter out `Without<DeployTimer>`. 
* **The Result:** If you drop a massive 3.0-second deploy Golem directly onto your own Archers, the physical footprint of the Golem will immediately shove the Archers out of the way, even while the Golem is "asleep" and loading in.
* **Action:** Keep this behavior. This perfectly mimics the official game's drop physics.