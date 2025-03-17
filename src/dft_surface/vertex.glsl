#version 130

precision mediump float;

#define PI 3.14159265358979323846264338327950288

attribute vec2 texcoord;
attribute vec3 position;

varying vec2 uv;
varying vec3 pos;

uniform vec2 surface_size;
uniform mat4 Model;
uniform mat4 Projection;

void main() {
	mat3 rotate = mat3(
		1.0, 0.0, 0.0,
		0.0, 1.0, 0.0,
		0.0, 0.0, 1.0
	);
	mat3 scale = mat3(
		1.0,  0.0, 0.0,
		0.0, -1.0, 0.0,
		0.0,  0.0, 1.0
	);
	vec3 translate = vec3(0.0, surface_size.y, 0.0);

	gl_Position = Projection * Model * vec4(rotate * scale * position + translate, 1.0);
	pos = position;
	uv = texcoord.xy;
}
