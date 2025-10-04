use macroquad::prelude::*;

//==============================================================================
// ESTRUTURAS DE DADOS BÁSICAS
//==============================================================================

/// Estrutura para representar um ponto 2D simples.
#[derive(Copy, Clone, Debug, PartialEq)]
struct P {
    x: f32,
    y: f32,
}
impl P {
    /// Construtor para um novo ponto.
    fn new(x: f32, y: f32) -> Self { Self { x, y } }

    /// Calcula o quadrado da distância euclidiana para outro ponto.
    fn dist2(&self, other: &P) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx*dx + dy*dy
    }
}

/// Estrutura para representar um triângulo, definido por três pontos.
#[derive(Clone, Debug)]
struct Triangle {
    a: P,
    b: P,
    c: P,
}

//==============================================================================
// FUNÇÕES AUXILIARES DE GEOMETRIA
//==============================================================================

/// Calcula a interseção de duas retas infinitas definidas pelos pontos p1-p2 e q1-q2.
/// Retorna "None" se as retas forem paralelas.
fn line_intersection(p1: P, p2: P, q1: P, q2: P) -> Option<P> {
    let a1 = p2.y - p1.y;
    let b1 = p1.x - p2.x;
    let c1 = a1*p1.x + b1*p1.y;

    let a2 = q2.y - q1.y;
    let b2 = q1.x - q2.x;
    let c2 = a2*q1.x + b2*q1.y;

    let det = a1*b2 - a2*b1;
    if det.abs() < 1e-6 { return None; } // Retas paralelas
    Some(P::new((b2*c1 - b1*c2)/det, (a1*c2 - a2*c1)/det))
}

/// Recorta um polígono (convexo ou não) por um semiplano.
/// O semiplano é definido por uma "normal" e um ponto "mid" na linha de corte.
/// Mantém os pontos que estão do lado positivo do semiplano (produto escalar >= 0).
/// Implementa uma variação do algoritmo de Sutherland-Hodgman.
fn clip_polygon_halfplane(poly: &Vec<P>, normal: P, mid: P) -> Vec<P> {
    let mut out: Vec<P> = Vec::new();
    if poly.is_empty() { return out; }

    let mut prev = *poly.last().unwrap();
    let mut prev_inside = ((prev.x - mid.x) * normal.x + (prev.y - mid.y) * normal.y) >= 0.0;

    for &cur in poly.iter() {
        let cur_inside = ((cur.x - mid.x) * normal.x + (cur.y - mid.y) * normal.y) >= 0.0;

        if prev_inside && cur_inside { // Ambos dentro: mantém o ponto atual
            out.push(cur);
        } else if prev_inside && !cur_inside { // Saindo do plano: adiciona a interseção
            if let Some(ix) = line_intersection(prev, cur,
                                                P::new(mid.x - normal.y, mid.y + normal.x),
                                                P::new(mid.x + normal.y, mid.y - normal.x)) {
                out.push(ix);
            }
        } else if !prev_inside && cur_inside { // Entrando no plano: adiciona a interseção e depois o ponto atual
            if let Some(ix) = line_intersection(prev, cur,
                                                P::new(mid.x - normal.y, mid.y + normal.x),
                                                P::new(mid.x + normal.y, mid.y - normal.x)) {
                out.push(ix);
            }
            out.push(cur);
        } // else: ambos fora, não faz nada
        prev = cur;
        prev_inside = cur_inside;
    }
    out
}

//==============================================================================
// DIAGRAMA DE VORONOI
//==============================================================================

/// Constrói a célula de Voronoi para um "site" (ponto) específico.
/// A célula é formada recortando um polígono de limites inicial
/// pelos semiplanos definidos pelas mediatrizes entre o "site" atual e todos os outros.
fn voronoi_cell(site_idx: usize, sites: &Vec<P>, bounds: &Vec<P>) -> Vec<P> {
    if site_idx >= sites.len() { return vec![]; }
    let site = sites[site_idx];
    let mut poly = bounds.clone();

    for (j, other) in sites.iter().enumerate() {
        if j == site_idx { continue; }

        // A mediatriz entre "site" e 'other' é o conjunto de pontos p onde (p - mid) . (other - site) = 0
        let mid = P::new((site.x + other.x)/2.0, (site.y + other.y)/2.0);
        let normal = P::new(other.x - site.x, other.y - site.y); // Normal aponta para 'other'

        // Queremos manter o lado mais próximo do nosso 'site', ou seja, onde (p - mid) . normal <= 0.
        // Como nossa função de recorte mantém o lado >= 0, invertemos a normal.
        let flip = P::new(-normal.x, -normal.y);
        poly = clip_polygon_halfplane(&poly, flip, mid);
        if poly.is_empty() { break; } // Se o polígono sumiu, não há mais o que recortar
    }
    poly
}

/// Triangula um polígono (assumido como convexo) para poder ser desenhado com preenchimento.
/// Usa uma técnica de "leque" (fan) a partir do centroide do polígono.
fn fill_polygon_triangles(poly: &Vec<P>) -> Vec<Triangle> {
    let n = poly.len();
    let mut tris = Vec::new();
    if n < 3 { return tris; }

    // Calcula o centroide
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    for p in poly { cx += p.x; cy += p.y; }
    cx /= n as f32; cy /= n as f32;
    let center = P::new(cx, cy);

    // Cria triângulos do centro para cada aresta do polígono
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i+1)%n]; // O operador '%' garante que o último ponto se conecte ao primeiro
        tris.push(Triangle { a: center, b: a, c: b });
    }
    tris
}

//==============================================================================
// TRIANGULAÇÃO DE DELAUNAY (ALGORITMO DE BOWYER-WATSON)
//==============================================================================

/// Verifica se um ponto "p" está dentro da circunferência circunscrita de um triângulo.
/// Calcula o circuncentro e o quadrado do raio para evitar o uso de raiz quadrada.
fn circumcircle_contains(tri: &Triangle, p: &P) -> bool {
    let ax = tri.a.x; let ay = tri.a.y;
    let bx = tri.b.x; let by = tri.b.y;
    let cx = tri.c.x; let cy = tri.c.y;
    let d = 2.0*(ax*(by-cy) + bx*(cy-ay) + cx*(ay-by));

    if d.abs() < 1e-6 { return false; } // Pontos colineares, não formam um círculo

    let ux = ((ax*ax+ay*ay)*(by-cy) + (bx*bx+by*by)*(cy-ay) + (cx*cx+cy*cy)*(ay-by)) / d;
    let uy = ((ax*ax+ay*ay)*(cx-bx) + (bx*bx+by*by)*(ax-cx) + (cx*cx+cy*cy)*(bx-ax)) / d;
    let dx = ux - p.x;
    let dy = uy - p.y;
    let r2 = (ux - ax)*(ux - ax) + (uy - ay)*(uy - ay);

    // Compara o quadrado da distância do ponto ao centro com o quadrado do raio
    dx*dx + dy*dy <= r2 + 1e-6
}

/// Implementa o algoritmo de Bowyer-Watson para gerar a triangulação de Delaunay.
fn bowyer_watson(sites: &Vec<P>) -> Vec<Triangle> {
    let mut triangles: Vec<Triangle> = Vec::new();
    if sites.len() < 2 { return triangles; }

    // --- 1. Inicialização: Cria um "super triângulo" que envolve todos os pontos ---
    let mut minx = sites[0].x; let mut maxx = sites[0].x;
    let mut miny = sites[0].y; let mut maxy = sites[0].y;
    for s in sites.iter() {
        if s.x < minx { minx = s.x; }
        if s.x > maxx { maxx = s.x; }
        if s.y < miny { miny = s.y; }
        if s.y > maxy { maxy = s.y; }
    }
    let dx = maxx - minx;
    let dy = maxy - miny;
    let delta_max = f32::max(dx.abs(), dy.abs()).max(1.0) * 10.0;
    let midx = (minx + maxx)/2.0;
    let midy = (miny + maxy)/2.0;

    let st_a = P::new(midx - 2.0*delta_max, midy - delta_max);
    let st_b = P::new(midx, midy + 2.0*delta_max);
    let st_c = P::new(midx + 2.0*delta_max, midy - delta_max);
    triangles.push(Triangle { a: st_a, b: st_b, c: st_c });

    // --- Loop principal: insere um ponto de cada vez ---
    for p in sites.iter() {
        // --- 2. Encontra todos os triângulos "ruins" cuja circunferência circunscrita contém o novo ponto ---
        let mut bad: Vec<usize> = Vec::new();
        for (i, tri) in triangles.iter().enumerate() {
            if circumcircle_contains(tri, p) {
                bad.push(i);
            }
        }

        // --- 3. Encontra a fronteira da cavidade poligonal formada pelos triângulos "ruins" ---
        let mut edges: Vec<(P,P)> = Vec::new();
        for &idx in bad.iter().rev() { // Itera em reverso para poder remover por índice
            let tri = &triangles[idx];
            let candidates = vec![(tri.a, tri.b), (tri.b, tri.c), (tri.c, tri.a)];
            for e in candidates {
                // Se a aresta já existe, ela é interna e deve ser removida. Se não, é uma borda.
                let mut found = false;
                let mut remove_idx = None;
                for (k, &existing) in edges.iter().enumerate() {
                    if (approx_eq(existing.0, e.0) && approx_eq(existing.1, e.1)) ||
                       (approx_eq(existing.0, e.1) && approx_eq(existing.1, e.0)) {
                        found = true; remove_idx = Some(k); break;
                    }
                }
                if found {
                    edges.remove(remove_idx.unwrap());
                } else {
                    edges.push(e);
                }
            }
        }

        // Remove os triângulos "ruins"
        for &idx in bad.iter().rev() {
            triangles.remove(idx);
        }

        // --- 4. Retriangula a cavidade formando novos triângulos ---
        for (ea, eb) in edges {
            triangles.push(Triangle { a: ea, b: eb, c: *p });
        }
    }

    // --- 5. Limpeza: Remove todos os triângulos que compartilham vértices com o super triângulo inicial ---
    triangles.into_iter()
        .filter(|t| {
            !uses_supervertex(&t.a, st_a, st_b, st_c) &&
            !uses_supervertex(&t.b, st_a, st_b, st_c) &&
            !uses_supervertex(&t.c, st_a, st_b, st_c)
        })
        .collect()
}

/// Função auxiliar para comparar dois pontos com uma pequena tolerância.
fn approx_eq(a: P, b: P) -> bool {
    ((a.x - b.x).abs() < 1e-3) && ((a.y - b.y).abs() < 1e-3)
}

/// Verifica se um ponto `p` é (aproximadamente) um dos vértices do super triângulo.
fn uses_supervertex(p: &P, a: P, b: P, c: P) -> bool {
    approx_eq(*p, a) || approx_eq(*p, b) || approx_eq(*p, c)
}

//==============================================================================
// APLICAÇÃO PRINCIPAL
//==============================================================================
#[macroquad::main("Voronoi")]
async fn main() {
    let mut sites: Vec<P> = Vec::new();

    loop {
        clear_background(Color::from_rgba(20, 20, 30, 255));

        let mouse = mouse_position();
        let mp = P::new(mouse.0, mouse.1);

        // --- Entrada do Usuário ---
        if is_mouse_button_pressed(MouseButton::Left) {
            sites.push(mp); // Adiciona um novo ponto com o botão esquerdo do mouse.
        }
        if is_key_pressed(KeyCode::C) {
            sites.clear(); // Limpa todos os pontos com a tecla 'C'.
        }
        if is_key_pressed(KeyCode::R) {
            // Remove o ponto mais próximo do mouse com a tecla 'R'.
            if !sites.is_empty() {
                let mut best_idx = 0usize;
                let mut best_dist_sq = sites[0].dist2(&mp);
                for (i, s) in sites.iter().enumerate().skip(1) {
                    let d = s.dist2(&mp);
                    if d < best_dist_sq { best_dist_sq = d; best_idx = i; }
                }
                // Só remove se estiver perto o suficiente
                if best_dist_sq < 40.0*40.0 { sites.remove(best_idx); }
            }
        }

        // Fecha a aplicação com Ctrl + C ou Esc.
        let ctrl_c_pressed = (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) && is_key_pressed(KeyCode::C);
        if ctrl_c_pressed || is_key_pressed(KeyCode::Escape) {
            break;
        }

        // --- Construção dos Limites ---
        // Define um polígono de limites (um retângulo grande) que engloba a tela.
        // Isso é usado como o polígono inicial para o recorte das células de Voronoi.
        let w = screen_width();
        let h = screen_height();
        let pad = 200.0;
        let bounds = vec![
            P::new(-pad, -pad),
            P::new(w+pad, -pad),
            P::new(w+pad, h+pad),
            P::new(-pad, h+pad),
        ];

        // --- Desenho das Células de Voronoi ---
        for (i, _) in sites.iter().enumerate() {
            let cell = voronoi_cell(i, &sites, &bounds);
            if cell.len() >= 3 {
                // Preenche com uma cor translúcida única para cada 'site'.
                let hue = ((i as f32 * 137.5) % 360.0) / 360.0;
                let color = hsv_to_rgba(hue, 0.5, 0.9, 0.12);

                // Triangula a célula (convexa) para poder preenchê-la com cor.
                let tris = fill_polygon_triangles(&cell);
                for tri in tris {
                    draw_triangle(
                        Vec2::new(tri.a.x, tri.a.y),
                        Vec2::new(tri.b.x, tri.b.y),
                        Vec2::new(tri.c.x, tri.c.y),
                        color
                    );
                }

                // Desenha a borda da célula.
                for j in 0..cell.len() {
                    let a = &cell[j];
                    let b = &cell[(j+1)%cell.len()];
                    draw_line(a.x, a.y, b.x, b.y, 1.0, Color::from_rgba(130,130,140,200));
                }
            }
        }

        // --- Desenho da Triangulação de Delaunay (Dual) ---
        // Calcula e desenha a triangulação de Delaunay, que é o grafo dual do diagrama de Voronoi.
        if sites.len() >= 2 {
            let triangles = bowyer_watson(&sites);
            for tri in triangles.iter() {
                draw_line(tri.a.x, tri.a.y, tri.b.x, tri.b.y, 1.5, WHITE);
                draw_line(tri.b.x, tri.b.y, tri.c.x, tri.c.y, 1.5, WHITE);
                draw_line(tri.c.x, tri.c.y, tri.a.x, tri.a.y, 1.5, WHITE);
            }
        }

        // --- Desenho dos Pontos ("sites") ---
        for (i, s) in sites.iter().enumerate() {
            draw_circle(s.x, s.y, 5.0, Color::from_rgba(240,240,240,255));
            // Desenha o índice do ponto ao lado dele.
            draw_text(&format!("{}", i), s.x + 6.0, s.y - 6.0, 14.0, Color::from_rgba(200,200,200,220));
        }

        // --- HUD (Interface do Usuário) ---
        let info = format!("Clique esquerdo: adicionar ponto   R: remover próximo   C: limpar   Ctrl+C/Esc: Sair   Pontos: {}", sites.len());
        draw_text(&info, 10.0, 20.0, 18.0, Color::from_rgba(220,220,220,220));

        next_frame().await
    }
}

/// Função auxiliar para converter cores do espaço HSV para RGBA (espera h em [0,1]).
fn hsv_to_rgba(h: f32, s: f32, v: f32, a: f32) -> Color {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r,g,b) = match i.rem_euclid(6) {
        0 => (v,t,p),
        1 => (q,v,p),
        2 => (p,v,t),
        3 => (p,q,v),
        4 => (t,p,v),
        _ => (v,p,q), // O caso 5 e o default
    };
    Color::new(r, g, b, a)
}
