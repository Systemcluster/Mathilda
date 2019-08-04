cbuffer GlobalBuffer: register(b0) {
	float4 subregion;
	float2 offset;
	float time;
	int mode;
};

struct Input {
	float4 fragCoord : SV_POSITION;
	float4 uv : TEXCOORD0;
};

struct Output {
	float4 color : SV_TARGET0;
};

#include "noise/util.hlsl"
#include "noise/curlnoise3d.hlsl"
#include "noise/snoise2d.hlsl"
#include "noise/snoise2dprd.hlsl"
#include "noise/snoise3d.hlsl"
#include "noise/snoise4d.hlsl"
#include "noise/pnoise2d.hlsl"
#include "noise/pnoise3d.hlsl"
#include "noise/pnoise4d.hlsl"
#include "noise/wnoise2d.hlsl"
#include "noise/wnoise3d.hlsl"
#include "noise/wnoise4d.hlsl"

Output main(Input input) {
	Output o;


	float4 seed = float4(
		time / 8000, 
		sin(time / 10000 / 100 - 100) * 100, 
		cos(time / 10000 / 100 + 100) * 100, 
		time / 12000
	);
	float2 position = input.uv.xy * 0.5 + 0.5;

	position = float2(position.x * (subregion.z - subregion.x) + subregion.x, position.y * (subregion.w - subregion.y) + subregion.y);
	position += offset;

	if (mode == 2) {
		float pos_mod = 0.005 * (subregion.z - subregion.x);
		position = trunc(position / pos_mod) * pos_mod;
	}

	float frequency = 4;
	float amplitude = 1;
	int iterations = 12;
	float lacunarity = 1.8;
	float persistence = 0.6;
	
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
	for(int i = 0; i < iterations; i+=1) {
		nv = pnoise4d(
			float4(nx, ny, nz, nw) * frequency
		);
		
		hv += nv * amplitude;
		
		amplitude_total += amplitude;
		
		amplitude *= persistence;
		frequency *= lacunarity;
	}
	hv /= amplitude_total;
	hv = hv * 0.5 + 0.5;
	hv = clamp(hv, 0.0, 1.0);


	float3 output = float3(hv);
	
	if (mode == 1 || mode == 2) {
		float3 c = output;
		// abyss
		if (hv <= 0.2) {
			c = c * float3(0.3, 0.4, 0.9);
		}
		// deep water
		else if (hv <= 0.4) {
			c = c * float3(0.5, 0.6, 0.9);
		}
		// water
		else if (hv <= 0.48) {
			c = c * float3(0.5, 0.7, 1.0);
		}
		// shallow water
		else if (hv <= 0.5) {
			c = c * float3(0.6, 0.75, 1.0);
		}
		// coastline
		else if (hv <= 0.52) {
			c = c * float3(0.6, 0.9, 0.6);
		}
		// flatland
		else if (hv <= 0.6) {
			c = c * float3(0.7, 1.0, 0.7);
		}
		// highland
		else if (hv <= 0.7) {
			c = c * float3(0.9, 0.8, 0.4);
		}
		// mountain
		else if (hv <= 0.8) {
			c = c * float3(0.9, 0.6, 0.4);
		}
		// peak
		else if (hv <= 1.0) {
			c = c * float3(0.9, 0.4, 0.4);
		}
		output = c;
	}

	float zone = sin(position.y * PI2 * 3 + PI/2);
	float pressure = (zone);
	float direction = abs(sin(position.y * PI2 + PI/2)) * 2 - 1;
	
	float speed = direction * (pressure * 0.5 + 0.5);
	
	if (mode == 3) {
		output = (float3(-1.0,  -1.0, speed) * 0.5 + 0.5);
	}

	if (mode == 4) {
		output = (float3(pressure, -1.0,  -1.0) * 0.5 + 0.5);
	}
	if (mode == 5) {

	}

	if (mode == 6) {
		output = float3(position.xy, 1);
	}
	// o.color = float4(hash(uint3(position.xy * 1000, 1.0)), 1.0);
	// o.color = float4(position, 1.0, 1.0);
	// o.color = float4(subregion.xyz, 1);
	
	o.color = float4(output, 1.0);
	return o;
}
