precision mediump float;

uniform float u_time;

// our texture
uniform sampler2D u_image;
 
// the texCoords passed in from the vertex shader.
varying vec2 v_texCoord;

void main() {
    float r = sin(u_time * 0.0003);
    float g = sin(u_time * 0.0005);
    float b = sin(u_time * 0.0007);

    gl_FragColor = texture2D(u_image, v_texCoord) * vec4(r, g, b, 1.0);
}
