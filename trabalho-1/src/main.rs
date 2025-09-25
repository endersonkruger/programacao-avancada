use macroquad::prelude::*;

/// Define a configuração inicial da janela da aplicação.
/// Retorna uma struct `Conf` com título e dimensões customizadas.
fn window_conf() -> Conf {
    Conf {
        window_title: "Trabalho 1".to_string(), // O título da janela
        window_width: 1280,                     // Largura desejada
        window_height: 720,                     // Altura desejada
        ..Default::default()                    // Usa valores padrão para as outras configurações
    }
}

/// Representa uma cor no formato RGBA (Vermelho, Verde, Azul, Alfa).
/// Usado para facilitar o armazenamento de cores nas geometrias.
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
    if n < 3 {
        return false;
    }
    for i in 0..n {
        let a = verts[i];
        let b = verts[(i + 1) % n]; // O operador '%' garante que a última aresta se conecte à primeira
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
    let t = ((p - a).dot(ab)) / (ab.dot(ab) + 1e-12); // Projeção do ponto no segmento
    let t_clamped = t.clamp(0.0, 1.0); // Garante que a projeção esteja dentro do segmento
    let proj = a + ab * t_clamped;
    (p - proj).length()
}

/// Ponto de entrada principal da aplicação.
/// O macro `#[macroquad::main(window_conf)]` inicializa o macroquad com as configurações da função `window_conf`.
#[macroquad::main(window_conf)]
async fn main() {
    // Inicializa o estado da aplicação. Começa com uma lista vazia de geometrias.
    let mut state = AppState {
        geometries: vec![],
        selected: None,
        drag_offset: Vec2::ZERO,
    };

    // Variáveis de controle para o estado da interface do usuário.
    let mut dragging = false;
    let mut hover_hint = String::new();

    // Loop principal do programa. Roda uma vez por frame.
    loop {
        // Limpa o fundo da tela a cada frame com uma cor cinza claro.
        clear_background(Color::new(0.95, 0.95, 0.95, 1.0));

        // Captura a posição atual do mouse.
        let (mx, my) = mouse_position();
        let mouse = vec2(mx, my);

        // --- SEÇÃO DE DESENHO ---
        // Itera sobre todas as geometrias no estado e as desenha na tela.
        for (i, geom) in state.geometries.iter().enumerate() {
            match geom {
                Geometry::Point { pos, color } => {
                    draw_circle(pos.x, pos.y, 6.0, (*color).into());
                    // Desenha um contorno se o ponto estiver selecionado.
                    if let Some((si, _)) = state.selected {
                        if si == i {
                            draw_circle(pos.x, pos.y, 9.0, Color::new(0.0, 0.0, 0.0, 0.15));
                        }
                    }
                }
                Geometry::Line {
                    a,
                    b,
                    color,
                    thickness,
                } => {
                    draw_line(a.x, a.y, b.x, b.y, *thickness, (*color).into());
                    // Desenha pequenos círculos nas extremidades da linha.
                    draw_circle(a.x, a.y, 4.0, (*color).into());
                    draw_circle(b.x, b.y, 4.0, (*color).into());
                    // Desenha um destaque no meio da linha se ela estiver selecionada.
                    if let Some((si, _)) = state.selected {
                        if si == i {
                            draw_circle(
                                (a.x + b.x) / 2.0,
                                (a.y + b.y) / 2.0,
                                10.0,
                                Color::new(0.0, 0.0, 0.0, 0.08),
                            );
                        }
                    }
                }
                Geometry::Polygon {
                    verts,
                    fill,
                    stroke,
                } => {
                    // Preenche o polígono desenhando triângulos a partir do primeiro vértice.
                    if verts.len() >= 3 {
                        let c = (*fill).into();
                        let first = verts[0];
                        for j in 1..(verts.len() - 1) {
                            let p1 = first;
                            let p2 = verts[j];
                            let p3 = verts[j + 1];
                            draw_triangle(p1, p2, p3, c);
                        }
                    }
                    // Desenha o contorno do polígono (as arestas).
                    for w in 0..verts.len() {
                        let a = verts[w];
                        let b = verts[(w + 1) % verts.len()];
                        draw_line(a.x, a.y, b.x, b.y, 2.0, (*stroke).into());
                    }
                    // Desenha os vértices do polígono.
                    for v in verts.iter() {
                        draw_circle(v.x, v.y, 4.0, (*stroke).into());
                    }
                    // Desenha uma caixa de contorno se o polígono estiver selecionado.
                    if let Some((si, _)) = state.selected {
                        if si == i {
                            draw_rectangle_lines(
                                verts.iter().map(|v| v.x).fold(f32::INFINITY, f32::min) - 6.0,
                                verts.iter().map(|v| v.y).fold(f32::INFINITY, f32::min) - 6.0,
                                verts.iter().map(|v| v.x).fold(-f32::INFINITY, f32::max)
                                    - verts.iter().map(|v| v.x).fold(f32::INFINITY, f32::min)
                                    + 12.0,
                                verts.iter().map(|v| v.y).fold(-f32::INFINITY, f32::max)
                                    - verts.iter().map(|v| v.y).fold(f32::INFINITY, f32::min)
                                    + 12.0,
                                2.0,
                                Color::new(0., 0., 0., 0.12),
                            );
                        }
                    }
                }
            }
        }

        // --- SEÇÃO DE DETECÇÃO DE HOVER (PICKING) ---
        // Verifica qual objeto está sob o cursor do mouse para seleção.
        hover_hint.clear();
        let mut picked: Option<(usize, Option<usize>)> = None;
        'outer: for (i, geom) in state.geometries.iter().enumerate() {
            match geom {
                Geometry::Point { pos, .. } => {
                    if (mouse - *pos).length() <= 8.0 {
                        hover_hint = format!("Point #{}", i);
                        picked = Some((i, None));
                        break 'outer;
                    }
                }
                Geometry::Line { a, b, .. } => {
                    if distance_point_segment(mouse, *a, *b) <= 6.0 {
                        hover_hint = format!("Line #{}", i);
                        picked = Some((i, None));
                        break 'outer;
                    }
                    if (mouse - *a).length() <= 8.0 {
                        hover_hint = format!("Line #{} endpoint A", i);
                        picked = Some((i, Some(0)));
                        break 'outer;
                    }
                    if (mouse - *b).length() <= 8.0 {
                        hover_hint = format!("Line #{} endpoint B", i);
                        picked = Some((i, Some(1)));
                        break 'outer;
                    }
                }
                Geometry::Polygon { verts, .. } => {
                    // Prioriza a seleção de vértices sobre a forma inteira.
                    for (vi, v) in verts.iter().enumerate() {
                        if (mouse - *v).length() <= 8.0 {
                            hover_hint = format!("Polygon #{} vertex {}", i, vi);
                            picked = Some((i, Some(vi)));
                            break 'outer;
                        }
                    }
                    if point_in_polygon(mouse, verts) {
                        hover_hint = format!("Polygon #{} (inside)", i);
                        picked = Some((i, None));
                        break 'outer;
                    }
                }
            }
        }

        // --- SEÇÃO DE DICAS DE UI ---
        // Desenha uma dica contextual perto do mouse se um objeto estiver em foco.
        if !hover_hint.is_empty() {
            draw_text(&hover_hint, mouse.x + 12.0, mouse.y + 4.0, 20.0, BLACK);
        } else {
            // Caso contrário, mostra a instrução padrão.
            draw_text(
                "Clique & Arraste para mover | Botao direito para desselecionar",
                12.0,
                22.0,
                20.0,
                DARKGRAY,
            );
        }

        // --- SEÇÃO DE MANIPULAÇÃO DE EVENTOS DO MOUSE ---
        // Lida com o clique inicial para selecionar e iniciar o arrasto.
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(p) = picked {
                state.selected = Some(p);
                dragging = true;
                // Calcula o offset para um arrasto suave.
                match state.selected {
                    Some((si, Some(vi))) => {
                        // Se um vértice foi selecionado
                        if let Geometry::Polygon { verts, .. } = &state.geometries[si] {
                            state.drag_offset = verts[vi] - mouse;
                        } else if let Geometry::Line { a, b, .. } = &state.geometries[si] {
                            let target = if vi == 0 { *a } else { *b };
                            state.drag_offset = target - mouse;
                        } else if let Geometry::Point { pos, .. } = &state.geometries[si] {
                            state.drag_offset = *pos - mouse;
                        } else {
                            state.drag_offset = Vec2::ZERO;
                        }
                    }
                    Some((si, None)) => {
                        // Se uma geometria inteira foi selecionada
                        state.drag_offset = match &state.geometries[si] {
                            Geometry::Point { pos, .. } => *pos - mouse,
                            Geometry::Line { a, b, .. } => ((*a + *b) * 0.5) - mouse,
                            Geometry::Polygon { verts, .. } => {
                                let center = verts.iter().fold(Vec2::ZERO, |acc, v| acc + *v)
                                    / (verts.len() as f32);
                                center - mouse
                            }
                        };
                    }
                    _ => {}
                }
            } else {
                // Se nada foi clicado, limpa a seleção.
                state.selected = None;
            }
        }

        // Lida com o movimento do objeto enquanto o botão do mouse está pressionado.
        if is_mouse_button_down(MouseButton::Left) && dragging {
            if let Some((si, maybe_vi)) = state.selected {
                match &mut state.geometries[si] {
                    Geometry::Point { pos, .. } => {
                        *pos = mouse + state.drag_offset;
                    }
                    Geometry::Line { a, b, .. } => {
                        if let Some(vi) = maybe_vi {
                            // Move um vértice
                            if vi == 0 {
                                *a = mouse + state.drag_offset;
                            } else {
                                *b = mouse + state.drag_offset;
                            }
                        } else {
                            // Move a linha inteira
                            let center = (*a + *b) * 0.5;
                            let delta = (mouse + state.drag_offset) - center;
                            *a += delta;
                            *b += delta;
                        }
                    }
                    Geometry::Polygon { verts, .. } => {
                        if let Some(vi) = maybe_vi {
                            // Move um vértice
                            verts[vi] = mouse + state.drag_offset;
                        } else {
                            // Move o polígono inteiro
                            let center = verts.iter().fold(Vec2::ZERO, |acc, v| acc + *v)
                                / (verts.len() as f32);
                            let delta = (mouse + state.drag_offset) - center;
                            for v in verts.iter_mut() {
                                *v += delta;
                            }
                        }
                    }
                }
            }
        }

        // Finaliza o estado de arrasto quando o botão do mouse é solto.
        if is_mouse_button_released(MouseButton::Left) {
            dragging = false;
        }

        // Desseleciona o objeto com o clique do botão direito.
        if is_mouse_button_pressed(MouseButton::Right) {
            state.selected = None;
            dragging = false;
        }

        // --- SEÇÃO DE ATALHOS DE TECLADO ---

        // CTRL + C -> Fecha a aplicação.
        if (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
            && is_key_pressed(KeyCode::C)
        {
            break; // Sai do loop principal, encerrando o programa.
        }

        // N -> Cria um novo polígono (triângulo) na posição do mouse.
        if is_key_pressed(KeyCode::N) {
            state.geometries.push(Geometry::Polygon {
                verts: vec![
                    mouse + vec2(0.0, -30.0),
                    mouse + vec2(-30.0, 20.0),
                    mouse + vec2(30.0, 20.0),
                ],
                fill: ColorRGBA(0.8, 0.6, 0.2, 0.5),
                stroke: ColorRGBA(0.0, 0.0, 0.0, 1.0),
            });
        }

        // P -> Cria um novo ponto na posição do mouse.
        if is_key_pressed(KeyCode::P) {
            state.geometries.push(Geometry::Point {
                pos: mouse,
                color: ColorRGBA(0.0, 0.7, 0.0, 1.0),
            });
        }

        // L -> Cria uma nova linha na posição do mouse.
        if is_key_pressed(KeyCode::L) {
            state.geometries.push(Geometry::Line {
                a: mouse,
                b: mouse + vec2(60.0, 20.0),
                color: ColorRGBA(0.0, 0.0, 0.0, 1.0),
                thickness: 3.0,
            });
        }

        // A -> Adiciona um vértice a um polígono selecionado.
        if is_key_pressed(KeyCode::A) {
            if let Some((si, None)) = state.selected {
                if let Geometry::Polygon { verts, .. } = &mut state.geometries[si] {
                    // Insere o novo vértice na aresta mais próxima do mouse.
                    let mut best_index = 0;
                    let mut min_dist = f32::MAX;
                    for i in 0..verts.len() {
                        let p1 = verts[i];
                        let p2 = verts[(i + 1) % verts.len()];
                        let dist = distance_point_segment(mouse, p1, p2);
                        if dist < min_dist {
                            min_dist = dist;
                            best_index = i + 1;
                        }
                    }
                    verts.insert(best_index, mouse);
                }
            }
        }

        // D -> Deleta um vértice selecionado de um polígono.
        if is_key_pressed(KeyCode::D) {
            if let Some((si, Some(vi))) = state.selected {
                let mut needs_deselect = false;
                if let Geometry::Polygon { verts, .. } = &mut state.geometries[si] {
                    // Impede a remoção se o polígono ficar com menos de 3 vértices.
                    if verts.len() > 3 {
                        verts.remove(vi);
                        needs_deselect = true;
                    }
                }
                if needs_deselect {
                    state.selected = None;
                    dragging = false;
                }
            }
        }

        // DELETE -> Apaga a geometria selecionada.
        if is_key_pressed(KeyCode::Delete) {
            if let Some((si, _)) = state.selected {
                state.geometries.remove(si);
                // Invalida a seleção, pois o índice pode estar incorreto.
                state.selected = None;
                dragging = false;
            }
        }

        // --- SEÇÃO DE TEXTOS DA UI FIXA ---
        // Desenha o texto de ajuda com os atalhos na parte inferior da tela.
        draw_text(
            "N: Triângulo P: Ponto L: Linha | A: Add Vértice D: Del Vértice | DEL: Apagar Forma | Ctrl + C: Sair",
            12.0,
            screen_height() - 12.0,
            20.0,
            DARKGRAY,
        );

        // Sinaliza o fim do frame atual e aguarda o próximo.
        next_frame().await;
    }
}
