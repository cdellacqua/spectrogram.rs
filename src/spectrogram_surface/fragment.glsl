#version 100

precision mediump float;

varying vec2 uv;

uniform sampler2D spectrogram;

void main() {
	gl_FragColor = texture2D(spectrogram, uv);
}
