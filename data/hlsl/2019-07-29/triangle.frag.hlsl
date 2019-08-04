struct Input {
	float4 fragCoord : SV_POSITION;
	float4 uv : TEXCOORD0;
};

struct Output {
	float4 color : SV_TARGET0;
};

#include "curlnoise3d.hlsl"
#include "psrdnoise2d.hlsl"
#include "noise/snoise2d.hlsl"
#include "noise/snoise3d.hlsl"
#include "noise/snoise4d.hlsl"
#include "noise/util.hlsl"



float uniformTest1;

Output main(Input input) {
	Output o;

	float2 pos = input.uv.xy;
	float2 offset = float2(14.5, 3.0) * (sin(uniformTest1/5000.0) * 0.5 + 0.5);
	float freq = 1.0;

	pos = pos + offset;
	float v;
	float4 g;
	// v = (curlnoise(float3(pos * freq, 0.5)) + 1.0) / 2.0;
	// v = snoise4d(input.uv * freq, g) * 0.5 + 0.5;
	// g = g * 0.5 + 0.5;

	freq = freq * 2.0;
	// v = snoise2d(pos * freq);
	v = psrnoise2d(pos * freq, float2(freq * 2), 0.2);
	// v = snoise3d(float3(pos * freq, pos.x));
	// v = snoise4d(float4(pos * freq, pos.x, pos.y));
	freq = freq * 2.0;
	// v += snoise2d(pos * freq) * 0.5;
	v += psrnoise2d(pos * freq, float2(freq * 2), 0.3) * 0.5;
	// v += snoise3d(float3(pos * freq, pos.x)) * 0.5;
	// v += snoise4d(float4(pos * freq, pos.x, pos.y)) * 0.5;
	freq = freq * 2.0;
	// v += snoise2d(pos * freq) * 0.25;
	v += psrnoise2d(pos * freq, float2(freq * 2), 0.4) * 0.25;
	// v += snoise3d(float3(pos * freq, pos.x)) * 0.25;
	// v += snoise4d(float4(pos * freq, pos.x, pos.y)) * 0.25;

	// v = curlnoise3d(float3(pos * freq, 0.5));
	// v = psnoise2d(pos * freq, float2(5, 5)) * 0.5 + 0.5;

	v = v * 0.45 + 0.5;
	v = clamp(v, 0, 1);

	v = inverse_smoothstep(v);
	// o.color = float4(v, v * 0.5, 1.0 - v, 1.0);
	// o.color = float4(g);
	// o.color = float4(hash(uint3((pos * 0.5 + 0.5) * 100, 1)), 1);
	// o.color = float4(input.uv.xy + 0.5, 0.0, 1.0);
	// o.color = clamp(o.color, 0.3, 1);
	o.color = float4(v, cos(uniformTest1/1000.0) * 0.5 + 0.5, 0.5, 1.0);
	return o;
}
