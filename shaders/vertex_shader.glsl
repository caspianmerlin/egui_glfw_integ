#version 330 core

layout (location = 0) in vec2 a_pos_coord;
layout (location = 1) in vec2 a_tex_coord;
layout (location = 2) in vec4 a_colour_srgba;

uniform vec2 u_screen_dimensions_pts;
uniform sampler2D u_sampler;

out vec2 v_tex_coord;
out vec4 v_rgba;


// 0-1 linear  from  0-255 sRGB
vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));


}

// 0-1 linear  from  0-255 sRGBA
vec4 linear_from_srgba(vec4 srgba) {
    return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main() {
    gl_Position = vec4(
        2.0 * a_pos_coord.x / u_screen_dimensions_pts.x - 1.0,
        1.0 - 2.0 * a_pos_coord.y / u_screen_dimensions_pts.y,
        0.0,
        1.0
    );
    v_tex_coord = a_tex_coord;
    v_rgba = linear_from_srgba(a_colour_srgba);
    v_rgba.a = pow(v_rgba.a, 1.6);
}







