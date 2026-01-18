## Updates and Improvements

### Need to create a win condition - Game / Set  / Match

### Need to add sound

### Need to add menu for players to pick colour and type of match

### Need to increase speed based on how long a key is held pressed.

----
# Mental Model and Code Explanation: Bevy + Rapier 2D Pong

This document explains the code in two layers:

1. **How the code works (system-by-system, data-by-data)**
2. **A mental model for re‑creating it from scratch**

The goal is not just to understand *what* this code does, but *how to think* in Bevy + Rapier so you can rebuild it incrementally.

---

## 1. High-Level Architecture (Mental Model First)

Think of this project as five cooperating subsystems:

1. **App & Engine Setup** – window, plugins, physics
2. **Entities & Components** – paddles, ball, walls, UI
3. **Physics & Collision** – Rapier bodies, colliders, sensors
4. **Gameplay Systems** – movement, scoring, reset logic
5. **Event Flow** – decoupling collisions from game logic

Bevy follows **ECS (Entity–Component–System)** strictly:

* **Entities** are IDs
* **Components** are data
* **Systems** are logic operating on data

No entity “does” anything. Systems *observe and mutate data* every frame.

---

## 2. App Setup (`main`)

### What happens here

```rust
fn main() {
    let mut app = App::new();
```

You are constructing a Bevy application as a **pipeline**.

### Window & Rendering

```rust
app.add_plugins(DefaultPlugins.set(WindowPlugin { ... }));
```

* Fixed resolution
* Non-resizable window
* Default Bevy rendering, input, assets, etc.

### Physics

```rust
app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
```

Adds Rapier’s simulation loop as a Bevy plugin.

```rust
app.insert_resource(RapierConfiguration {
    gravity: Vec2::ZERO,
    ..RapierConfiguration::new(1.0)
});
```

* No gravity (top‑down Pong)
* Physics timestep scaling

### Resources & Events

```rust
app.init_resource::<Score>();
app.add_event::<GameEvents>();
```

* **Resources** = global state (`Score`)
* **Events** = messages between systems

### Systems & Scheduling

```rust
app.add_systems(Startup, (...));
app.add_systems(Update, (...));
app.add_systems(PostUpdate, (...));
```

Mental model:

| Schedule   | Purpose              |
| ---------- | -------------------- |
| Startup    | Spawn entities       |
| Update     | Game logic per frame |
| PostUpdate | React to events      |

---

## 3. Core Components (Data Only)

### Paddle

```rust
#[derive(Component)]
struct Paddle {
    move_up: KeyCode,
    move_down: KeyCode,
}
```

* Input mapping per paddle
* No logic here

### Player Enum

```rust
enum Player { Player1, Player2 }
```

This is used as:

* Paddle owner
* Goal owner
* Score key
* Ball reset direction

This is **identity**, not behavior.

Helper methods:

* `start_speed()` → initial ball velocity
* `get_colour()` → visual identity

---

## 4. Spawning the World (Startup Systems)

### Borders & Goals

```rust
fn spawn_border(...)
```

Four invisible physics objects:

| Object       | Purpose     | Notes                |
| ------------ | ----------- | -------------------- |
| Top / Bottom | Ball bounce | Fixed rigid body     |
| Left / Right | Goals       | Sensors + Player tag |

Sensors:

* Detect collisions
* Do NOT affect physics

This is critical for scoring.

---

### Camera

```rust
fn spawn_camera(...)
```

Simple 2D camera.

---

### Players (Paddles)

```rust
RigidBody::KinematicPositionBased
```

Mental model:

* **You control position directly**
* Physics responds, but you drive movement

Each paddle has:

* Sprite (visual)
* Collider (physics)
* Paddle (input mapping)
* Player (identity)

---

### Ball

```rust
RigidBody::Dynamic
Velocity::linear(...)
```

Mental model:

* Physics controls movement
* You only influence it via forces / velocity

Important components:

| Component         | Why                         |
| ----------------- | --------------------------- |
| ActiveEvents      | Enables collision detection |
| CollidingEntities | Tracks current overlaps     |
| Restitution       | Bounciness                  |

---

## 5. Frame-by-Frame Gameplay Systems

### Paddle Movement

```rust
fn move_paddle(...)
```

Key ideas:

* Query = all paddles
* Read input
* Apply movement scaled by `delta_seconds`
* Clamp to window bounds

This is **pure ECS**:

> For every entity with (Transform + Paddle), apply input logic.

---

### Ball Color Change on Hit

```rust
fn ball_hit(...)
```

Logic:

* Read current collisions
* Check if collided entity is a paddle
* Change sprite color

No physics modification here—just visuals.

---

## 6. Events as the Backbone

### Detect Reset & Score

```rust
fn detect_reset(...)
```

This system does **detection only**:

* Space bar → reset
* Ball hits goal sensor → reset + score

It does NOT mutate the ball or score directly.

Instead:

```rust
game_events.send(...)
```

This keeps systems decoupled.

---

### Reset Ball

```rust
fn reset_ball(...)
```

Responds to events:

* Move ball to center
* Set velocity based on player

Only runs in `PostUpdate`, after detection.

---

### Scoring UI

```rust
fn score(...)
```

Flow:

1. Receive `GainPoint`
2. Update `Score` resource
3. Update matching UI text entity

The UI text entities are tagged with `Player`, allowing filtering.

---

## 7. UI Spawning

```rust
fn spawn_score(...)
```

Mental model:

* UI is just entities too
* Hierarchy matters
* Text is data, not special logic

The `Player` tag links UI text to game logic.

---

## 8. How to Rebuild This From Scratch (Recommended Order)

### Step-by-step approach

1. **Empty window + camera**
2. **One paddle (keyboard movement)**
3. **Second paddle**
4. **Static walls**
5. **Ball with physics + bounce**
6. **Collision detection**
7. **Events for reset**
8. **Score resource + UI**

At each step:

> Ask: *What data does this need? What system mutates it?*

---

## 9. Key Bevy + Rapier Mental Rules

* Components are dumb data
* Systems do all behavior
* Physics ≠ game rules
* Events decouple cause and effect
* Identity is often a component (`Player`)

If you internalize this, you will be able to write this code again block‑by‑block without memorization.


------
## How to use this to build a game
### When starting a new Bevy/Rapier project define your workflow elements:

* `Entities`: What objects are in my game? (Player, Enemy, Bullet, Wall).

* Physics Role: For each entity, what is it?

* Does gravity/impacts move it? -> `Dynamic`.

* Do I move it via inputs, but it pushes others? -> `Kinematic`.

* Is it scenery? -> `Fixed`.

* Is it an invisible trigger zone? -> `Sensor`.

* Data: What data does each need? (Health, Ammo count, Speed stats). Create `Components` for these.

* Action Systems: What inputs change things before physics runs? (Movement, shooting).

* Reaction Systems: What happens because of physics collisions? (Taking damage, triggering a cutscene, scoring a point). Use `Sensors` and `CollidingEntities` heavily here.

* Events: When a reaction happens, don't handle it immediately. Send an `event` (e.g., `Event::UnitDied`, `Event::ItemCollected`) and write a separate system to handle the fallout.