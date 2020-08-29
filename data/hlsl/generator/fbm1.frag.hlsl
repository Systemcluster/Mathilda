#pragma shader_stage(fragment)

cbuffer GlobalBuffer : register(b0) {
	float4 subregion;
	float2 offset;
	float time;
	int mode;
	float level;
};

struct Input {
	float4 fragCoord : SV_POSITION;
	float4 uv : TEXCOORD0;
};

struct Output {
	float4 value : SV_TARGET0;
};

#include "noise/util.hlsl"
#include "noise/curlnoise3d.hlsl"
#include "noise/pnoise2d.hlsl"
#include "noise/pnoise3d.hlsl"
#include "noise/pnoise4d.hlsl"
#include "noise/snoise2d.hlsl"
#include "noise/snoise2dprd.hlsl"
#include "noise/snoise3d.hlsl"
#include "noise/snoise4d.hlsl"
#include "noise/wnoise2d.hlsl"
#include "noise/wnoise3d.hlsl"
#include "noise/wnoise4d.hlsl"
#include "noise/osnoise4d.hlsl"

Output main(Input input) {
	Output o;

	float4 seed = float4(
		time / 12000,
		sin(time / 10000 / 100 - 100) * 100,
		cos(time / 10000 / 100 + 100) * 100,
		time / 12000
	) + 0.025;

	float2 position = input.uv.xy * 0.5 + 0.5;
	position = float2(
		position.x * (subregion.z - subregion.x) + subregion.x,
		position.y * (subregion.w - subregion.y) + subregion.y
	);
	position += offset;
	if (mode == 2) {
		float pos_mod = 0.005 * (subregion.z - subregion.x);
		position = trunc(position / pos_mod) * pos_mod;
	}

	float frequency = 4;
	float amplitude = 1;
	int iterations = 12;
	float lacunarity = 1.8;
	float persistence = 0.62;
	float exponent = 1.25;

	float amplitude_total = 0;

	static const float PI2 = 6.283185307179586476925286766559f;
	static const float PI = PI2 / 2.0;
	static const float
		x1 = 0.0,
		y1 = 0.0,
		x2 = 1.0,
		y2 = 1.0;
	static const float
		dxp = (x2 - x1) / PI2,
		dyp = (y2 - y1) / PI2;
	float
		ox = seed.x + x1,
		oy = seed.y + y1,
		oz = seed.z + x2,
		ow = seed.w + y2;
	float
		s = position.x,
		t = position.y;
	float
		nx = ox + cos(s * PI2) * dxp,
		ny = oy + cos(t * PI2) * dyp,
		nz = oz + sin(s * PI2) * dxp,
		nw = ow + sin(t * PI2) * dyp;

	float hv = 0;
	float nv = 0;
	for (int i = 0; i < iterations; i += 1) {
		nv = pnoise4d(float4(nx, ny, nz, nw) * frequency);
		nv = clamp(nv * 1.4, -1.0, 1.0); // normalize to [-1, +1]
		hv += nv * amplitude;
		amplitude_total += amplitude;
		amplitude *= persistence;
		frequency *= lacunarity;
	}
	hv = hv / amplitude_total;
	hv = pow(abs(hv * 2), exponent) * sign(hv) * 0.5; // exponentiate
	hv = clamp(hv * 0.5 + 0.5, 0.0, 1.0); // transform to [0, 1]

	o.value = float4(hv, 0, 0, 0);
	return o;
}
