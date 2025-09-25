use std::fs::File;
use std::io::Write;
use std::time::Instant;
use macroquad::prelude::*;

/// Define a configuração inicial da janela da aplicação.
fn window_conf() -> Conf {
    Conf {
        window_title: "Trabalho 1".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

/// Representa uma cor no formato RGBA (Vermelho, Verde, Azul, Alfa).
#[derive(Clone, Copy)]
struct ColorRGBA(f32, f32, f32, f32);

/// Permite a conversão direta de `ColorRGBA` para a struct `Color` do macroquad.
impl From<ColorRGBA> for Color {
    fn from(c: ColorRGBA) -> Self {
        Color::new(c.0, c.1, c.2, c.3)
    }
}

/// Enumeração que define todos os tipos de geometrias suportadas pela aplicação.
#[derive(Clone)]
enum Geometry {
    /// Um ponto 2D com uma posição e cor.
    Point { pos: Vec2, color: ColorRGBA },
    /// Uma linha definida por dois pontos (a, b), com cor e espessura.
    Line {
        a: Vec2,
        b: Vec2,
        color: ColorRGBA,
        thickness: f32,
    },
    /// Um polígono definido por uma lista de vértices, com cores de preenchimento e contorno.
    Polygon {
        verts: Vec<Vec2>,
        fill: ColorRGBA,
        stroke: ColorRGBA,
    },
}

/// Estrutura para registrar as interações do usuário, como movimento do mouse e cliques.
struct Logger {
    mouse_path: Vec<(f64, f32, f32)>, // (tempo_decorrido, x, y)
    click_count: usize,
    clicks: Vec<ClickEvent>,
}

/// Representa um único evento de clique do mouse.
struct ClickEvent {
    time: f64,
    geom_index: Option<usize>,
    geom_kind: String,
    pos: (f32, f32),
}

impl Logger {
    /// Cria uma nova instância do Logger.
    fn new() -> Self {
        Self {
            mouse_path: Vec::new(),
            click_count: 0,
            clicks: Vec::new(),
        }
    }

    /// Registra a posição do mouse em um determinado momento.
    fn log_mouse(&mut self, t: f64, x: f32, y: f32) {
        self.mouse_path.push((t, x, y));
    }

    /// Registra um evento de clique, incluindo qual geometria foi atingida (se houver).
    fn log_click(&mut self, t: f64, geom_index: Option<usize>, geom_kind: String, x: f32, y: f32) {
        self.click_count += 1;
        self.clicks.push(ClickEvent {
            time: t,
            geom_index,
            geom_kind,
            pos: (x, y),
        });
    }

    /// Salva todo o log de eventos em um arquivo de texto.
    fn save_to_file(&self, started: Instant, filename: &str) {
        let elapsed = started.elapsed().as_secs_f64();
        let mut s = String::new();

        // Seção de Resumo
        s.push_str(&format!("Execution time (s): {:#.6}\n", elapsed));
        s.push_str(&format!("Total clicks: {}\n\n", self.click_count));

        // Seção do Percurso do Mouse
        s.push_str("Mouse path (time_seconds, x, y):\n");
        for (t, x, y) in &self.mouse_path {
            s.push_str(&format!("{:#.6}, {}, {}\n", t, x, y));
        }

        // Seção de Eventos de Clique
        s.push_str("\nClick events (time_seconds, geom_index, geom_kind, x, y):\n");
        for c in &self.clicks {
            s.push_str(&format!(
                "{:#.6}, {:?}, {}, {}, {}\n",
                c.time, c.geom_index, c.geom_kind, c.pos.0, c.pos.1
            ));
        }

        if let Ok(mut f) = File::create(filename) {
            if let Err(e) = f.write_all(s.as_bytes()) {
                eprintln!("Erro ao escrever no arquivo de log: {}", e);
            }
        } else {
            eprintln!("Erro: Não foi possível criar o arquivo de log '{}'", filename);
        }
    }
}

/// Contém todo o estado da aplicação.
/// Esta struct é a "memória" do programa, guardando todos os objetos e seleções.
struct AppState {
    /// Um vetor que armazena todas as geometrias atualmente na tela.
    geometries: Vec<Geometry>,
    /// Guarda o item selecionado: `Some((índice_da_geometria, Some(índice_do_vértice)))`.
    /// Se o segundo valor for `None`, a geometria inteira está selecionada.
    selected: Option<(usize, Option<usize>)>,
    /// Armazena o deslocamento do mouse em relação ao objeto ao iniciar o arrasto.
    drag_offset: Vec2,
}

/// Verifica se um ponto está dentro de um polígono usando o algoritmo de Ray-Casting.
fn point_in_polygon(pt: Vec2, verts: &[Vec2]) -> bool {
    let mut inside = false;
    let n = verts.len();
    if n < 3 { return false; }
    for i in 0..n {
        let a = verts[i];
        let b = verts[(i + 1) % n];
        let (ay, by) = (a.y, b.y);
        let (ax, bx) = (a.x, b.x);
        let intersect = ((ay > pt.y) != (by > pt.y))
            && (pt.x < (bx - ax) * (pt.y - ay) / (by - ay + 1e-12) + ax);
        if intersect {
            inside = !inside;
        }
    }
    inside
}

/// Calcula a menor distância entre um ponto `p` e um segmento de reta definido por `a` e `b`.
fn distance_point_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab)) / (ab.dot(ab) + 1e-12);
    let t_clamped = t.clamp(0.0, 1.0);
    let proj = a + ab * t_clamped;
    (p - proj).length()
}

/// Ponto de entrada principal da aplicação.
#[macroquad::main(window_conf)]
async fn main() {
    let mut state = AppState {
        geometries: vec![],
        selected: None,
        drag_offset: Vec2::ZERO,
    };

    let started = Instant::now();
    let mut logger = Logger::new();

    let mut dragging = false;
    let mut hover_hint = String::new();

    // Loop principal do programa, executa uma vez por frame.
    loop {
        clear_background(Color::new(0.95, 0.95, 0.95, 1.0));

        let (mx, my) = mouse_position();
        let mouse = vec2(mx, my);

        // Registra a posição do mouse a cada frame para o log.
        let elapsed = started.elapsed().as_secs_f64();
        logger.log_mouse(elapsed, mx, my);

        // Desenho de Geometrias
        for (i, geom) in state.geometries.iter().enumerate() {
            match geom {
                Geometry::Point { pos, color } => {
                    draw_circle(pos.x, pos.y, 6.0, (*color).into());
                    if let Some((si, _)) = state.selected {
                        if si == i { draw_circle(pos.x, pos.y, 9.0, Color::new(0.0, 0.0, 0.0, 0.15)); }
                    }
                }
                Geometry::Line { a, b, color, thickness, } => {
                    draw_line(a.x, a.y, b.x, b.y, *thickness, (*color).into());
                    draw_circle(a.x, a.y, 4.0, (*color).into());
                    draw_circle(b.x, b.y, 4.0, (*color).into());
                    if let Some((si, _)) = state.selected {
                        if si == i { draw_circle( (a.x + b.x) / 2.0, (a.y + b.y) / 2.0, 10.0, Color::new(0.0, 0.0, 0.0, 0.08), ); }
                    }
                }
                Geometry::Polygon { verts, fill, stroke, } => {
                    if verts.len() >= 3 {
                        let c = (*fill).into();
                        let first = verts[0];
                        for j in 1..(verts.len() - 1) {
                            draw_triangle(first, verts[j], verts[j + 1], c);
                        }
                    }
                    for w in 0..verts.len() {
                        let a = verts[w];
                        let b = verts[(w + 1) % verts.len()];
                        draw_line(a.x, a.y, b.x, b.y, 2.0, (*stroke).into());
                    }
                    for v in verts.iter() {
                        draw_circle(v.x, v.y, 4.0, (*stroke).into());
                    }
                    if let Some((si, _)) = state.selected {
                        if si == i {
                            let (min_x, max_x) = verts.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| (min.min(v.x), max.max(v.x)));
                            let (min_y, max_y) = verts.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| (min.min(v.y), max.max(v.y)));
                            draw_rectangle_lines( min_x - 6.0, min_y - 6.0, (max_x - min_x) + 12.0, (max_y - min_y) + 12.0, 2.0, Color::new(0., 0., 0., 0.12), );
                        }
                    }
                }
            }
        }

        // Detecção de objeto sob o mouse (Picking)
        hover_hint.clear();
        let mut picked: Option<(usize, Option<usize>)> = None;
        'outer: for (i, geom) in state.geometries.iter().enumerate() {
            match geom {
                Geometry::Point { pos, .. } => {
                    if (mouse - *pos).length() <= 8.0 {
                        hover_hint = format!("Ponto #{}", i);
                        picked = Some((i, None));
                        break 'outer;
                    }
                }
                Geometry::Line { a, b, .. } => {
                    if (mouse - *a).length() <= 8.0 {
                        hover_hint = format!("Linha #{} (ponta A)", i);
                        picked = Some((i, Some(0)));
                        break 'outer;
                    }
                    if (mouse - *b).length() <= 8.0 {
                        hover_hint = format!("Linha #{} (ponta B)", i);
                        picked = Some((i, Some(1)));
                        break 'outer;
                    }
                    if distance_point_segment(mouse, *a, *b) <= 6.0 {
                        hover_hint = format!("Linha #{}", i);
                        picked = Some((i, None));
                        break 'outer;
                    }
                }
                Geometry::Polygon { verts, .. } => {
                    for (vi, v) in verts.iter().enumerate() {
                        if (mouse - *v).length() <= 8.0 {
                            hover_hint = format!("Polígono #{}, Vértice {}", i, vi);
                            picked = Some((i, Some(vi)));
                            break 'outer;
                        }
                    }
                    if point_in_polygon(mouse, verts) {
                        hover_hint = format!("Polígono #{}", i);
                        picked = Some((i, None));
                        break 'outer;
                    }
                }
            }
        }

        // Lógica de Eventos do Mouse
        if is_mouse_button_pressed(MouseButton::Left) {
            let click_time = started.elapsed().as_secs_f64();
            if let Some((idx, _)) = picked {
                let kind = match &state.geometries[idx] {
                    Geometry::Point { .. } => "Point",
                    Geometry::Line { .. } => "Line",
                    Geometry::Polygon { .. } => "Polygon",
                }.to_string();
                logger.log_click(click_time, Some(idx), kind, mx, my);
            } else {
                logger.log_click(click_time, None, "None".to_string(), mx, my);
            }

            if let Some(p) = picked {
                state.selected = Some(p);
                dragging = true;
                match state.selected {
                    Some((si, Some(vi))) => {
                        if let Geometry::Polygon { verts, .. } = &state.geometries[si] {
                            state.drag_offset = verts[vi] - mouse;
                        } else if let Geometry::Line { a, b, .. } = &state.geometries[si] {
                            state.drag_offset = if vi == 0 { *a } else { *b } - mouse;
                        } else if let Geometry::Point { pos, .. } = &state.geometries[si] {
                            state.drag_offset = *pos - mouse;
                        }
                    }
                    Some((si, None)) => {
                        state.drag_offset = match &state.geometries[si] {
                            Geometry::Point { pos, .. } => *pos - mouse,
                            Geometry::Line { a, b, .. } => ((*a + *b) * 0.5) - mouse,
                            Geometry::Polygon { verts, .. } => {
                                let center = verts.iter().fold(Vec2::ZERO, |a, v| a + *v) / (verts.len() as f32);
                                center - mouse
                            }
                        };
                    }
                    _ => {}
                }
            } else {
                state.selected = None;
            }
        }

        if is_mouse_button_down(MouseButton::Left) && dragging {
            if let Some((si, maybe_vi)) = state.selected {
                match &mut state.geometries[si] {
                    Geometry::Point { pos, .. } => *pos = mouse + state.drag_offset,
                    Geometry::Line { a, b, .. } => {
                        if let Some(vi) = maybe_vi {
                            if vi == 0 { *a = mouse + state.drag_offset; } else { *b = mouse + state.drag_offset; }
                        } else {
                            let center = (*a + *b) * 0.5;
                            let delta = (mouse + state.drag_offset) - center;
                            *a += delta; *b += delta;
                        }
                    }
                    Geometry::Polygon { verts, .. } => {
                        if let Some(vi) = maybe_vi {
                            verts[vi] = mouse + state.drag_offset;
                        } else {
                            let center = verts.iter().fold(Vec2::ZERO, |a, v| a + *v) / (verts.len() as f32);
                            let delta = (mouse + state.drag_offset) - center;
                            for v in verts.iter_mut() { *v += delta; }
                        }
                    }
                }
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            dragging = false;
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            state.selected = None;
            dragging = false;
        }

        // Atalhos de Teclado
        let ctrl_c_pressed = (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) && is_key_pressed(KeyCode::C);
        if ctrl_c_pressed || is_key_pressed(KeyCode::Escape) {
            logger.save_to_file(started, "mouse_log.txt");
            println!("Log salvo em 'mouse_log.txt'. Saindo...");
            break;
        }

        if is_key_pressed(KeyCode::N) {
            state.geometries.push(Geometry::Polygon {
                verts: vec![ mouse + vec2(0.0, -30.0), mouse + vec2(-30.0, 20.0), mouse + vec2(30.0, 20.0) ],
                fill: ColorRGBA(0.8, 0.6, 0.2, 0.5),
                stroke: ColorRGBA(0.0, 0.0, 0.0, 1.0),
            });
        }
        if is_key_pressed(KeyCode::P) {
            state.geometries.push(Geometry::Point { pos: mouse, color: ColorRGBA(0.0, 0.7, 0.0, 1.0) });
        }
        if is_key_pressed(KeyCode::L) {
            state.geometries.push(Geometry::Line { a: mouse, b: mouse + vec2(60.0, 20.0), color: ColorRGBA(0.0, 0.0, 0.0, 1.0), thickness: 3.0 });
        }
        if is_key_pressed(KeyCode::A) {
            if let Some((si, None)) = state.selected {
                if let Geometry::Polygon { verts, .. } = &mut state.geometries[si] {
                    let mut best_index = 0;
                    let mut min_dist = f32::MAX;
                    for i in 0..verts.len() {
                        let p1 = verts[i];
                        let p2 = verts[(i + 1) % verts.len()];
                        let dist = distance_point_segment(mouse, p1, p2);
                        if dist < min_dist { min_dist = dist; best_index = i + 1; }
                    }
                    verts.insert(best_index, mouse);
                }
            }
        }
        if is_key_pressed(KeyCode::D) {
            if let Some((si, Some(vi))) = state.selected {
                if let Geometry::Polygon { verts, .. } = &mut state.geometries[si] {
                    if verts.len() > 3 {
                        verts.remove(vi);
                        state.selected = None;
                        dragging = false;
                    }
                }
            }
        }
        if is_key_pressed(KeyCode::Delete) {
            if let Some((si, _)) = state.selected {
                state.geometries.remove(si);
                state.selected = None;
                dragging = false;
            }
        }

        // Dicas de Interface (UI)
        if !hover_hint.is_empty() {
            draw_text(&hover_hint, mouse.x + 12.0, mouse.y + 4.0, 20.0, BLACK);
        } else {
            draw_text( "Clique & Arraste para mover | Botão direito para desselecionar", 12.0, 22.0, 20.0, DARKGRAY, );
        }
        draw_text( "N: Polígono, P: Ponto, L: Linha | A: Add Vértice, D: Rem Vértice | DEL: Apagar | ESC/Ctrl+C: Sair e Salvar Log", 12.0, screen_height() - 12.0, 20.0, DARKGRAY, );

        // Avança para o próximo frame.
        next_frame().await;
    }
}
