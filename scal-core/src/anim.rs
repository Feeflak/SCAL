use crate::{AnimOP, Time};

/// Wait for some time and do nothing :|
/// Everyone needs some rest.
#[must_use]
pub const fn wait(duration: Time) -> AnimOP {
    AnimOP::Wait(duration, None)
}

/// Execute Animations at once.
///```
///            parallel![
///                cw.add_lines()
///                    .str(
///                        r"
///fn fib(n: u32) -> u32 {
///    match n {
///        0 => 0,
///        1 => 1,
///        _ => fib(n - 1) + fib(n - 2),
///    }
///}
///                "
///                    )
///                    .over(5.s())
///                    .style(CodeAnimationStyle::TypeWriterInstantResize),
///                typing.play(),
///            ],
///```
#[macro_export]
macro_rules! parallel {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::All($crate::timeline![ $( $op ),* ], None)
    };
}

/// Allows you to do a sequence of animations inside of a parallel block
///```
///parallel![
///    sequence![
///        cw.close_button().scale().to(Vec2::ONE * 0.8).over(0.3.s()),
///        cw.close_button().scale().to(Vec2::ONE).over(0.3.s()),
///    ],
///    click.play(),
///],
///```
#[macro_export]
macro_rules! sequence {
    ( $( $op:expr ),* $(,)? ) => {
        $crate::AnimOP::Sequence($crate::timeline![ $( $op ),* ], None)
    };
}
