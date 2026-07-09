
BRO WTF IS THIS SHIT:             AnimOP::CodeAddLines(code.id, NEW_LINES.to_string(), 1, 5.s(), Ease::Linear, CodeAnimationStyle::TypeWriter)???
IT'S EVEN WORSE THEN BEFORE THE REWORK!
make it work like any other animation, so:
code.add_lines()
.str(r#"...."#)
.over(2.5.s())
.ease(...)
.style(...)
make the defualt ease linear and style type writer

# Implement these for all the creators
instead of:
rect()
..
create(position);

make the position be another function of the creators, and do the same for rotation and scale-
rect()
.color(...)
.pos(vec2(1,2))
.z(1)
.scale(vec2(2,1))
.rot(1.deg())
.build()


--
Instead of things like:

rect.position(vec2(...))

do

rect.position()
    .object(...)
    .to(vec2(...))
    .over(1.s()) 

#  Move all examples to the new ipc api
#  Remove the legacy run loop code.
