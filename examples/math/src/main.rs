use glam::{Vec2, vec2};
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let red = color(1.0, 0.42, 0.42, 1.0);
    let blue = color(0.30, 0.67, 0.97, 1.0);
    let green = color(0.41, 0.86, 0.49, 1.0);
    let marker = color(1.0, 0.90, 0.25, 1.0);
    let violet = color(0.75, 0.55, 1.0, 1.0);

    // A math style bundles font colors and the highlight look, so each equation only says what
    // is different. Variables an equation does not use are skipped.
    let style = math_style()
        .vars([("a", red), ("b", blue), ("c", green)])
        .highlight(marker, 0.2);
    let recolor = Motion::new(1.0, Ease::InOutCubic);

    let title = text()
        .spans([
            span("Math with "),
            span("per-symbol").color(red).bold(),
            span(" colors"),
        ])
        .font_size(64.0)
        .pos(vec2(960.0, 130.0))
        .build();

    // Named colors: `\textcolor{a}{..}` picks `.var_color("a", ..)`. Named parts can be animated.
    let quadratic = math(r"\textcolor{a}{a}x^2 + \textcolor{b}{b}x + \textcolor{c}{c} = 0")
        .style(&style)
        .font_size(110.0)
        .pos(vec2(960.0, 320.0))
        .build();

    // Every occurrence of a part is animated together.
    let solution = math(
        r"x = \frac{-\textcolor{b}{b} \pm \sqrt{\textcolor{b}{b}^2 - 4\,\textcolor{a}{a}\,\textcolor{c}{c}}}{2\,\textcolor{a}{a}}",
    )
    .style(&style)
    .font_size(100.0)
    .pos(vec2(960.0, 600.0))
    .build();

    // Plain LaTeX colors also work (they are fixed), and `latex_color` formats a `Color`.
    let euler = math(format!(
        r"e^{{i\pi}} + 1 = 0 \quad \textcolor{{{}}}{{\int_0^1 x^2\,dx = \frac{{1}}{{3}}}}",
        latex_color(color(1.0, 0.85, 0.3, 1.0))
    ))
    .font_size(64.0)
    .pos(vec2(960.0, 900.0))
    .build();

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.03, 0.03, 0.04, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            spawn![title, quadratic, solution, euler],
            wait(1.0.s()),
            // Highlighting tints the parts and dims everything else. It stays until you
            // highlight something else (a crossfade) or reset it. Every `a` is hit together.
            parallel![
                quadratic.highlight(["a"]).over(0.6),
                solution.highlight(["a"]).over(0.6),
            ],
            wait(1.0.s()),
            parallel![
                quadratic.highlight(["b"]).over(0.6),
                solution.highlight(["b"]).over(0.6),
            ],
            wait(1.0.s()),
            // Several parts at once.
            parallel![
                quadratic.highlight(["a", "c"]).over(0.6),
                solution.highlight(["a", "c"]).over(0.6),
            ],
            wait(1.0.s()),
            parallel![
                quadratic.reset_highlight().over(0.6.s()),
                solution.reset_highlight().over(0.6.s()),
            ],
            wait(0.5.s()),
            // A plain recolor of one part stays until you change it back.
            quadratic.part("a").color().to(marker).over(0.5.s()),
            wait(0.5.s()),
            // Only the uncolored parts follow the object color; the named parts keep theirs.
            parallel![
                quadratic.color().to(violet).with(recolor),
                solution.color().to(violet).with(recolor),
                euler.scale_to(1.15).with(recolor),
            ],
            wait(0.5.s()),
            // Fading affects the whole equation, parts included.
            parallel![
                quadratic.color().to(Color { a: 0.0, ..violet }).over(1.0),
                solution.color().to(Color { a: 0.0, ..violet }).over(1.0),
            ],
            wait(0.5.s()),
        ],
    }
}
