# Manuale Di Codifica Manuale

Questa guida spiega come modificare a mano i file di Vibe Engine senza perdersi nella struttura del progetto. Il codice è piccolo, ma è diviso in moduli per separare gameplay, rendering, UI, shader e generazione procedurale.

## Regole Di Base

- Esegui `cargo fmt` dopo aver modificato file Rust.
- Esegui `cargo check` per controllare rapidamente se il codice compila.
- Esegui `cargo run` quando vuoi provare davvero la finestra e il gioco.
- Modifica un sistema alla volta: gameplay in `game.rs`, grafica in `renderer.rs`, mesh generate in `procedural.rs`, menu in `ui.rs`.
- Se aggiungi un nuovo file `.rs` in `src`, dichiaralo in alto in `src/main.rs` con `mod nome_file;`.

## Mappa Dei File

### `src/main.rs`

Questo file avvia l'applicazione e contiene il ciclo eventi principale.

Modifica questo file quando vuoi:

- Aggiungere una nuova schermata o modalità, per esempio `Pause`, `Settings` o `GameOverMenu`.
- Cambiare i controlli da tastiera o mouse.
- Cambiare il comportamento del cursore quando entri o esci dal gameplay.
- Cambiare titolo finestra o risoluzione iniziale.
- Decidere quale funzione di rendering viene chiamata per ogni stato dell'app.

Sezioni importanti:

- `mod app; mod config; ...`: dichiarazioni dei moduli.
- `enter_game(...)`: passaggio dal menu al gameplay.
- `enter_menu(...)`: ritorno dal gameplay al menu.
- `WindowEvent::KeyboardInput`: gestione tasti.
- `WindowEvent::MouseInput`: gestione click mouse.
- `WindowEvent::RedrawRequested`: ciclo di update e disegno.

Esempio: cambiare titolo finestra:

```rust
let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
    .with_title("Il Mio Gioco Spaziale")
    .with_inner_size(1280, 720)
    .build(&event_loop);
```

Esempio: aggiungere un nuovo tasto:

```rust
PhysicalKey::Code(KeyCode::KeyP) if pressed => {
    // Qui puoi gestire la pausa dopo aver aggiunto lo stato Paused.
}
```

### `src/app.rs`

Questo file contiene lo stato generale dell'applicazione.

Modifica questo file quando vuoi:

- Aggiungere una schermata a `AppState`.
- Aggiungere nuovi flag di input a `Input`.
- Cambiare la posizione iniziale del mouse o lo stato input predefinito.

Stati attuali:

- `MainMenu`
- `Playing`

Esempio: aggiungere uno stato pausa:

```rust
pub enum AppState {
    MainMenu,
    Playing,
    Paused,
}
```

Dopo aver aggiunto uno stato, aggiorna `main.rs` per dire al ciclo eventi come renderizzarlo e come entrarci/uscirci.

### `src/config.rs`

Questo file contiene costanti condivise di gameplay e rendering.

Modifica questo file per cambiamenti rapidi di bilanciamento:

- `WORLD_HALF_WIDTH`: metà larghezza dell'area giocabile.
- `WORLD_HALF_HEIGHT`: metà altezza dell'area giocabile.
- `PLAYER_Z`: profondità del giocatore.
- `BULLET_SPEED`: velocità dei proiettili.
- `ASTEROID_SPEED`: velocità base degli asteroidi.
- `STAR_LAYERS`: numero di livelli parallasse delle stelle.
- `STAR_COUNT_PER_LAYER`: numero di stelle per livello.

Esempio: rendere il gioco più veloce:

```rust
pub const BULLET_SPEED: f32 = 34.0;
pub const ASTEROID_SPEED: f32 = 11.0;
```

Cambiare le costanti è di solito il primo intervento più sicuro, perché raramente richiede modifiche in altri file.

### `src/game.rs`

Questo file contiene la simulazione del gioco.

Modifica questo file quando vuoi:

- Cambiare la vita del giocatore.
- Cambiare il cooldown dei colpi.
- Cambiare lo spawn dei nemici.
- Aggiungere nuovi comportamenti nemici.
- Cambiare le collisioni.
- Cambiare il punteggio.
- Aggiungere powerup o oggetti raccoglibili.

Struct importanti:

- `Game`: stato completo della simulazione.
- `Player`: posizione, bersaglio, cooldown e vita della nave.
- `Bullet`: posizione del proiettile.
- `Asteroid`: posizione, velocità, raggio, rotazione e mesh scelta.

Funzioni importanti:

- `Game::new()`: stato iniziale del gioco.
- `Game::update(...)`: simulazione eseguita ogni frame.

Esempio: cambiare vita iniziale:

```rust
health: 8,
```

Esempio: cambiare cadenza di fuoco:

```rust
self.player.cooldown = 0.08;
```

Un cooldown più basso significa sparare più rapidamente.

Esempio: far spawnare meno asteroidi:

```rust
self.spawn_timer = self.rng.gen_range(0.45..1.05) / difficulty.min(3.5);
```

Esempio: cambiare punti per asteroide:

```rust
self.score += 25;
```

Quando modifichi `Game::update`, mantieni idealmente questo ordine:

1. Gestire restart dopo game over.
2. Aggiornare bersaglio e movimento del giocatore.
3. Generare proiettili.
4. Muovere proiettili.
5. Generare asteroidi.
6. Muovere asteroidi.
7. Risolvere collisioni proiettile-asteroide.
8. Risolvere collisioni giocatore-asteroide.
9. Rimuovere entità distrutte o fuori schermo.

Questo ordine rende la simulazione più prevedibile.

### `src/graphics.rs`

Questo file contiene tipi condivisi per il rendering.

Modifica questo file quando vuoi:

- Aggiungere attributi ai vertici.
- Cambiare cosa contiene ogni vertice.
- Aggiungere comportamento alle trasformazioni.
- Cambiare la creazione delle mesh GPU.

Struct importanti:

- `Vertex`: posizione, normale, colore.
- `StarVertex`: posizione, colore, dimensione.
- `Mesh`: vertex buffer e index buffer sulla GPU.
- `Transform`: posizione, rotazione, scala.

Fai attenzione quando modifichi `Vertex` o `StarVertex`. Se aggiungi campi, devi anche:

- Aggiornare `implement_vertex!(...)`.
- Aggiornare tutto il codice che crea vertici.
- Aggiornare gli input GLSL in `src/shaders.rs`.

Esempio: aggiungere coordinate UV richiederebbe modifiche in `graphics.rs`, `procedural.rs`, `ui.rs` e `shaders.rs`.

### `src/procedural.rs`

Questo file genera mesh e dati procedurali.

Modifica questo file quando vuoi:

- Cambiare forma della nave del giocatore.
- Cambiare ruvidità o densità poligonale degli asteroidi.
- Cambiare forma dei proiettili.
- Cambiare distribuzione, luminosità, dimensione o profondità delle stelle.
- Aggiungere nuove funzioni di generazione mesh.

Funzioni importanti:

- `procedural_ship(...)`: crea i vertici della nave.
- `ship_indices()`: lista triangoli della nave.
- `bullet_mesh()`: vertici dei proiettili.
- `procedural_asteroid(...)`: vertici e indici degli asteroidi.
- `generate_stars(...)`: punti della starfield.
- `quad_mesh(...)`: rettangolo usato da UI e HUD.

Esempio: cambiare colore accento della nave:

```rust
let accent = [
    rng.gen_range(0.8..1.0),
    rng.gen_range(0.2..0.4),
    rng.gen_range(0.2..0.4),
];
```

Esempio: rendere gli asteroidi più irregolari:

```rust
let rough = rng.gen_range(0.55..1.45);
```

Esempio: rendere le stelle più grandi:

```rust
size: rng.gen_range(2.5..6.0) + layer as f32,
```

Importante: gli indici delle mesh puntano ai vertici nell'array. Se aggiungi o rimuovi vertici in `procedural_ship`, aggiorna `ship_indices()` in modo che ogni triangolo usi numeri validi.

### `src/renderer.rs`

Questo file disegna tutto con OpenGL tramite `glium`.

Modifica questo file quando vuoi:

- Cambiare posizione della camera o campo visivo.
- Cambiare ordine di disegno.
- Cambiare direzione della luce.
- Cambiare HUD.
- Cambiare colori o layout del menu principale.
- Aggiungere rendering per nuovi tipi di entità.
- Aggiungere nuovi programmi shader.

Funzioni importanti:

- `Renderer::new(...)`: carica shader e crea mesh GPU.
- `Renderer::render(...)`: disegna il gameplay.
- `Renderer::render_menu(...)`: disegna il menu principale.
- `draw_stars(...)`: disegna le stelle parallasse.
- `draw_mesh(...)`: disegna mesh 3D.
- `draw_hud(...)`: disegna vita e punteggio.
- `draw_ui_rect(...)`, `draw_menu_button(...)`, `draw_text(...)`: helper per UI e menu.

Esempio: cambiare camera gameplay:

```rust
let view = Matrix4::look_at_rh(
    Point3::new(0.0, -0.15, 12.0),
    Point3::new(0.0, -0.35, -15.0),
    Vector3::unit_y(),
);
```

Un valore `z` più alto nella camera la sposta più indietro.

Esempio: cambiare campo visivo:

```rust
let projection = perspective(Deg(65.0), aspect, 0.1, 160.0);
```

Esempio: cambiare titolo menu:

```rust
self.draw_text_centered(
    display,
    frame,
    ui,
    "IL MIO GIOCO",
    0.0,
    2.0,
    0.115,
    7.7,
    [0.45, 0.95, 1.0],
);
```

Se aggiungi una nuova entità in `game.rs`, disegnala in `Renderer::render(...)` dopo aver creato o scelto una mesh.

### `src/ui.rs`

Questo file contiene geometria menu e testo bitmap.

Modifica questo file quando vuoi:

- Spostare pulsanti del menu.
- Cambiare hitbox dei pulsanti.
- Cambiare conversione coordinate mouse/UI.
- Cambiare generazione del testo bitmap.
- Aggiungere caratteri supportati dal font integrato.

Elementi importanti:

- `MenuButton`: ID dei pulsanti menu.
- `Rect`: rettangolo UI e hit testing.
- `ui_projection(...)`: camera ortografica per UI.
- `mouse_to_ui(...)`: converte coordinate mouse normalizzate in coordinate UI.
- `menu_button(...)`: rettangoli dei pulsanti.
- `text_width(...)`: misura larghezza testo.
- `text_height(...)`: misura altezza testo.
- `text_mesh(...)`: converte testo in mesh di quad.
- `glyph(...)`: pattern bitmap 5x7.

Esempio: spostare il pulsante Start verso l'alto:

```rust
MenuButton::Start => Rect::new(-1.7, -0.25, 3.4, 0.62),
```

Esempio: aggiungere un nuovo pulsante menu:

```rust
pub enum MenuButton {
    Start,
    Settings,
    Quit,
}
```

Poi aggiorna `menu_button(...)`, `renderer.rs` e la gestione click in `main.rs`.

Esempio: aggiungere supporto per `!`:

```rust
'!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
```

### `src/shaders.rs`

Questo file contiene il codice sorgente GLSL degli shader.

Modifica questo file quando vuoi:

- Cambiare illuminazione.
- Cambiare rendering delle stelle.
- Aggiungere nuovi input o uniform agli shader.
- Cambiare colori o alpha nel fragment shader.

Shader attuali:

- `SOLID_VERTEX_SHADER`: trasforma vertici e normali delle mesh.
- `SOLID_FRAGMENT_SHADER`: illumina superfici colorate.
- `STAR_VERTEX_SHADER`: anima posizione e dimensione delle stelle.
- `STAR_FRAGMENT_SHADER`: rende le stelle morbide e circolari.

Esempio: rendere più luminose le mesh:

```glsl
vec3 color = v_color * (0.38 + diffuse * 1.1) + rim;
```

Esempio: rendere le stelle più nette:

```glsl
float alpha = smoothstep(0.18, 0.01, dist);
```

Se aggiungi una uniform nello shader, aggiungila anche nel blocco `uniform! { ... }` corrispondente in `renderer.rs`.

## Ricette Comuni

### Rendere Il Giocatore Più Reattivo

Apri `src/game.rs` e trova:

```rust
self.player.position +=
    (self.player.target - self.player.position) * (1.0 - (-14.0 * dt).exp());
```

Aumenta `14.0` per movimento più rapido. Diminuiscilo per movimento più morbido e pesante.

### Allargare L'Area Giocabile

Apri `src/config.rs`:

```rust
pub const WORLD_HALF_WIDTH: f32 = 10.5;
```

Poi esegui `cargo run` e verifica che nave e asteroidi restino bilanciati.

### Aggiungere Un Nuovo Nemico

Passi consigliati:

1. Aggiungi una struct o un campo enum in `src/game.rs`.
2. Generala dentro `Game::update`.
3. Aggiorna il movimento dentro `Game::update`.
4. Aggiungi collisioni.
5. Aggiungi una mesh procedurale in `src/procedural.rs`.
6. Crea la mesh GPU in `Renderer::new`.
7. Disegnala in `Renderer::render`.

Per la prima versione tienila semplice: posizione, velocità, raggio, mesh ID.

### Aggiungere Un Menu Pausa

Passi consigliati:

1. Aggiungi `Paused` a `AppState` in `src/app.rs`.
2. In `src/main.rs`, usa `P` o `Escape` per entrare/uscire dalla pausa.
3. In `WindowEvent::RedrawRequested`, non chiamare `game.update(...)` durante la pausa.
4. Aggiungi `Renderer::render_pause(...)` in `src/renderer.rs`, oppure riusa gli helper del menu.

### Aggiungere Un Pulsante Settings

Passi consigliati:

1. Aggiungi `Settings` a `MenuButton` in `src/ui.rs`.
2. Aggiungi un rettangolo per `Settings` in `menu_button(...)`.
3. Disegnalo in `Renderer::render_menu(...)`.
4. Gestisci il click in `src/main.rs`.
5. Aggiungi `Settings` a `AppState` se serve una schermata separata.

### Cambiare Testi Del Menu

Apri `src/renderer.rs` e modifica il testo passato a `draw_text_centered(...)` o `draw_text_in_rect(...)`.

Il font bitmap attuale supporta lettere maiuscole e numeri. Le minuscole vengono convertite in maiuscole. La punteggiatura non supportata diventa un glifo tipo punto interrogativo, a meno che tu non la aggiunga in `glyph(...)` dentro `src/ui.rs`.

### Cambiare La Starfield

Apri `src/procedural.rs`:

- Il numero di stelle è in `src/config.rs`.
- Distribuzione e profondità sono in `generate_stars(...)`.
- Velocità animazione stelle è in `draw_stars(...)` dentro `src/renderer.rs`.

Apri `src/shaders.rs` se vuoi cambiare forma o alpha dei punti stella.

## Aggiungere Nuovi File

Per aggiungere un nuovo modulo:

1. Crea `src/audio.rs`, `src/settings.rs` o un altro file con uno scopo preciso.
2. Aggiungi in alto a `src/main.rs`:

```rust
mod audio;
```

3. Importa gli elementi dove servono:

```rust
use audio::AudioSystem;
```

4. Marca come `pub` gli elementi che devono essere usati da altri file.

Regola pratica: se un altro file deve usare un tipo o una funzione, quell'elemento deve essere `pub`.

## Visibilità Rust In Breve

- `fn helper()` è privata al modulo corrente.
- `pub fn helper()` può essere usata da altri moduli.
- `struct Thing { field: i32 }` ha campi privati.
- `pub struct Thing { pub field: i32 }` può essere costruita e letta da altri moduli.
- `pub enum Mode { A, B }` può essere usata in `match` da altri moduli.

Tieni privati gli helper interni. Rendi pubblica solo la vera API del modulo.

## Workflow Di Compilazione E Debug

Usa questo ciclo mentre modifichi:

```powershell
cargo fmt
cargo check
cargo run
```

Usa spesso `cargo check`, perché è più veloce di una build completa.

Per dettagli extra in caso di panic:

```powershell
$env:RUST_BACKTRACE=1
cargo run
```

Per rimuovere il backtrace nello stesso terminale:

```powershell
Remove-Item Env:RUST_BACKTRACE
```

## Errori Comuni

### `cannot find function/type in this scope`

Probabilmente hai aggiunto o spostato qualcosa ma non lo hai importato.

Correzione tipica:

```rust
use crate::ui::menu_button;
```

Dentro `main.rs`, i moduli si importano senza `crate::`:

```rust
use ui::menu_button;
```

### `field is private`

Il campo della struct non è `pub`.

Esempio:

```rust
pub struct Player {
    pub health: i32,
}
```

Rendi pubblici i campi solo quando un altro modulo deve davvero leggerli o modificarli.

### `UniformTypeMismatch`

Il tipo Rust della uniform non corrisponde al tipo GLSL.

Esempi:

- GLSL `float` richiede Rust `f32`.
- GLSL `vec3` richiede `[f32; 3]`.
- GLSL `mat4` richiede `[[f32; 4]; 4]`.

Corretto:

```rust
layer_speed: 3.5f32,
light_dir: [0.35f32, 0.8, 0.5],
vp: mat4(vp),
```

### Mesh Invisibile O Vuota

Controlla:

- Gli indici sono validi per l'array di vertici?
- L'oggetto è dietro la camera?
- L'oggetto è troppo piccolo o troppo grande?
- Il colore è troppo scuro?
- Il depth test lo sta nascondendo?
- Il nome dell'input shader corrisponde al campo Rust del vertice?

### Un Carattere Del Testo Non Si Vede

Aggiungi il carattere in `glyph(...)` dentro `src/ui.rs`, oppure usa lettere e numeri già supportati.

## Linee Guida Di Stile

- Tieni la simulazione in `game.rs`.
- Tieni il rendering OpenGL in `renderer.rs`.
- Tieni geometria generata in `procedural.rs`.
- Tieni valori di bilanciamento condivisi in `config.rs`.
- Tieni gli shader in `shaders.rs`.
- Tieni hitbox UI e testo bitmap in `ui.rs`.
- Evita decisioni di gameplay negli shader.
- Evita creazione buffer rendering dentro `game.rs`.
- Mantieni `main.rs` concentrato su eventi finestra e transizioni di stato.

Questa divisione rende le modifiche manuali più piccole, leggibili e facili da testare.
