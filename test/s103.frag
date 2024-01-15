in loc(0) let normal: vec3;
in loc(1) let fragColor: vec3;
out loc(0) let outColor: vec4;

const lightDir: vec3 = vec3(1., 1., 1.);

fragment entry fn main() -> void {
    let lightAmt = clamp(dot(-lightDir, normal), 0.2, 1.0);

    outColor = vec4(fragColor * lightAmt, 1.0);
}