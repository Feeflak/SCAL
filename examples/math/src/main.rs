use glam::{Vec2, vec2};
use scal_core::prelude::*;

#[scal_ipc::main]
fn main() -> Project {
    let red = color(1.0, 0.42, 0.42, 1.0);
    let blue = color(0.30, 0.67, 0.97, 1.0);
    let green = color(0.41, 0.86, 0.49, 1.0);

    let title = text()
        .spans([
            span("Math with "),
            span("per-symbol").color(red).bold(),
            span(" colors"),
        ])
        .font_size(64.0)
        .pos(vec2(960.0, 130.0))
        .build();

    // Named colors from the palette: `\textcolor{a}{..}` picks `.var_color("a", ..)`.
    let quadratic = math(r"\textcolor{a}{a}x^2 + \textcolor{b}{b}x + \textcolor{c}{c} = 0")
        .var_color("a", red)
        .var_color("b", blue)
        .var_color("c", green)
        .font_size(110.0)
        .pos(vec2(960.0, 320.0))
        .build();
    let quadratic_handle = MathHandle(quadratic.id);

    // Whole sub-expressions can be colored; uncolored parts follow the object color.
    let solution = math(
        r"x = \frac{-\textcolor{b}{b} \pm \sqrt{\textcolor{b}{b}^2 - 4\,\textcolor{a}{a}\,\textcolor{c}{c}}}{2\,\textcolor{a}{a}}",
    )
    .var_color("a", red)
    .var_color("b", blue)
    .var_color("c", green)
    .font_size(100.0)
    .pos(vec2(960.0, 600.0))
    .build();
    let solution_handle = MathHandle(solution.id);

    // Plain LaTeX colors also work, and `latex_color` formats a `Color` for `format!`.
    let euler = math(format!(
        r"e^{{i\pi}} + 1 = 0 \quad \textcolor{{{}}}{{\int_0^1 x^2\,dx = \frac{{1}}{{3}}}}",
        latex_color(color(1.0, 0.85, 0.3, 1.0))
    ))
    .font_size(64.0)
    .pos(vec2(960.0, 900.0))
    .build();
    let euler_handle = MathHandle(euler.id);

    Project {
        scene_settings: SceneSettings {
            background_color: color(0.03, 0.03, 0.04, 1.0),
            camera: Camera::new(Vec2::new(1920., 1080.), Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            title.instantiate(),
            quadratic.instantiate(),
            solution.instantiate(),
            euler.instantiate(),
            wait(1.0.s()),
            // Only the uncolored parts (x, ^2, +, =, 0, ...) change; a, b, c keep their colors.
            parallel![
                quadratic_handle
                    .color()
                    .to(color(0.75, 0.55, 1.0, 1.0))
                    .over(1.0.s())
                    .ease(Ease::InOutCubic),
                solution_handle
                    .color()
                    .to(color(0.75, 0.55, 1.0, 1.0))
                    .over(1.0.s())
                    .ease(Ease::InOutCubic),
                euler_handle
                    .scale()
                    .to(vec2(1.15, 1.15))
                    .over(1.0.s())
                    .ease(Ease::InOutCubic),
            ],
            wait(1.0.s()),
            // Fades affect the whole equation, colored symbols included.
            parallel![
                quadratic_handle
                    .color()
                    .to(color(0.75, 0.55, 1.0, 0.0))
                    .over(1.0.s()),
                solution_handle
                    .color()
                    .to(color(0.75, 0.55, 1.0, 0.0))
                    .over(1.0.s()),
            ],
            wait(0.5.s()),
        ],
    }
}
