#version 100
precision mediump float;

uniform vec2 screen_size;
uniform vec2 spectrogram_size;
varying vec2 uv;
uniform sampler2D spectrogram;

void main() {
	gl_FragColor = texture2D(spectrogram, uv);
}
