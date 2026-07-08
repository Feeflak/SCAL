
# Implement as separate crates

## Spearate SCAL-runtime: 
The scal runtime is an separate cli app that handles the animation process, rendering, encoding, audio, graphical preview, etc. 
It runs the animation it self and recompiles upon change. 
It gets rendering and encoding setting by reading the Config.toml file in the animation directory: 
pub struct RenderingSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub buffer_count: u32,
    pub text_resolution_multiplier: f32,
}
pub struct EncodingSettings {
    pub output_path: String,
    pub codec_type: CodecType,
}
You sellect between render and preview by running this in the panimation directory:
scal render/preview.

It gets animation operations, after each animation rebuild, by ipc as a single Project struct encoded using bincode.

Preview freature will be implemented later.


## Separate SCAL-core:
It should only hold types for the api-AnimOps AnimObjects, Color, Project, AnimCurves, ... .
It should be as thin as it can be so it compiles really fast.

## SCAL-ipc
Used by the animation to communicate with the runtime.
It should implement a single macro like this:
#[scala_ipc::main]
that will do all the communications for you, so you just can do this in the animation.

#[scala_ipc::main]
pub fn main()-> Project{
...

Project{
scene_settings:SceneSettings{
background_color: Color,
camera:Camera
}
timeline:vec![
...
];
}
}


# Animation API changes

## The Main Function Just Returns a Project

It should look smth like this:


#[scala_ipc::main]
pub fn main()-> Project{
...

Project{
scene_settings:SceneSettings{
background_color: Color,
camera:Camera
}
timeline:vec![
...
];
}
}

## macros instead of bulky functions 

instead of `all(vec![...])` do `parallel!{...}`, the same for sequence. 

## Durations should read like English

Instead of

1.0
0.3
5.0

I'd add extension traits.

1.s()
300.ms()
5.s()

So

wait(1.s())

pointer.position_to(..., 500.ms(), Ease::OutBack)

is immediately readable.

## Curves should be builder methods

Instead of

scale_to(
    target,
    1.s(),
    AnimationCurve::EaseOutBack,
)


do 

pointer
    .scale(target)
    .over(1.s())
    .ease(Ease::OutBack)

Now every animation has the same API.
And you can easily have default values

## Objects are too verbose

Currently

pointer.transform().position_to(...)

appears everywhere.

I'd expose common transforms directly.

pointer.position(...)

pointer.scale(...)

pointer.scale(...)

Internally they still forward to transform().

## play(click, 0.)

The second parameter isn't obvious.

play(click)

or

play(click).after(200.ms())

is much clearer.

## rework the move function

do this:
pointer.position(vec2(25,25)).object(smth)
// Rest is normal
.over(1.s())
.ease(....)

This allows to combine the move and move to object functions.
The object is optional line the over and ease functions.

## Group object creation

Instead of

let pointer = svg(...);

let mut cw = code_window(...);

I'd like to see

let pointer =svg()
    .path()
    .size(40.)
    .color(WHITE)
    .create();

let code = code_window() 
    .theme(THEME)
    .line_numbers(true)
    .create();

Builder APIs work really well for objects.
Those functions should give a creator type that whould generate the final sturct after the .create().
This will make the setting more explicit and allow for default values.

## Audio

Instead of

play(click)

do

click.play().over(...);


