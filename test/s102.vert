let positions: vec2[3] = [
    vec2(0, -1),
    vec2(1, 1),
    vec2(-1, 1)
];

let colors: vec3[3] = [
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
];

out loc(0) let fragColor: vec3;

vertex entry fn main() -> void {
    SL_position = vec4(position[SL_vertex_index], 0., 1.);
    fragColor = colors[SL_vertex_index];
}