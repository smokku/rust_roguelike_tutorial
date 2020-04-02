#version 330 core
out vec4 FragColor;

in vec3 ourColor;
in vec2 TexCoord;
in vec3 ourBackground;

// texture sampler
uniform sampler2D texture1;

void main()
{
    vec4 original = texture(texture1, TexCoord);
    vec4 fg = original.r > 0.1f || original.g > 0.1f || original.b > 0.1f ? original * vec4(ourColor, 1.f) : vec4(ourBackground, 1.f);
	FragColor = fg;
}
