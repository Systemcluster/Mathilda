#version 450

#extension GL_GOOGLE_include_directive : require
#include "psrdnoise2d.glsl"

layout(location = 0) out vec4 outColor;

const uint k = 1664525U;
vec3 hash( uvec3 x )
{
    x = ((x>>8U)^x.yzx)*k;
    x = ((x>>8U)^x.yzx)*k;
    x = ((x>>8U)^x.yzx)*k;    
    return vec3(x)*(1.0/float(0xffffffffU));
}


void main() {
    uvec3 p = uvec3(gl_FragCoord.xy, 1);
    outColor = vec4(hash(p), 1.0);
}
