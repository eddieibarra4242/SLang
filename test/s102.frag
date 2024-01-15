in loc(0) let fragColor: vec3;
out loc(0) let outColor: vec4;

fragment entry fn main() -> void {
    outColor = vec4(fragColor, 1.0);
}