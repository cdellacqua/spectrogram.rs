#version 130

precision mediump float;

varying vec2 uv;
varying vec3 pos;

uniform vec2 surface_size;
uniform float max_dB;
uniform float min_dB;
uniform sampler2D dft;

float rgba_to_f32(vec4 color) {
  vec4 bytes = color * 255.0;
  float sign = round(1.0 - 2.0 * step(128.0, bytes.r));
  float exponent = round(2.0 * mod(bytes.r, 128.0) + step(128.0, bytes.g)); 
  float fraction = round(mod(bytes.g, 128.0) * 65536.0 + bytes.b * 256.0 + bytes.a);

  if (exponent == 0.0 && fraction == 0.0) {
    return sign * 0.0;
  }

  return sign * exp2(exponent - 127.0) * (1.0 + exp2(-23.0) * fraction); 
}

void main() {
	vec4 sample = texture2D(dft, uv);
	float dB = rgba_to_f32(sample);

  float range = max_dB - min_dB;
  float ratio = min((dB - min_dB) / range, 1.0);
  vec4 low    = vec4(0.0, 0.0, 0.2, 1.0);
  vec4 high   = vec4(1.0, 1.0, 1.0, 1.0);

  if (ratio * surface_size.y > pos.y) {
    gl_FragColor = high;
  } else {
    gl_FragColor = low;
  }
}
