cbuffer GlobalBuffer: register(b0) {
	float4 subregion;
	float2 offset;
	float time;
	int mode;
};
Texture2D<float> map : register(t1);
sampler sam : register(s2);

struct Input {
	float4 fragCoord : SV_POSITION;
	float4 uv : TEXCOORD0;
};

struct Output {
	float4 color : SV_TARGET0;
};

Output main(Input input) {
	Output o;

	float hv = map.Sample(sam, input.uv * 0.5 + 0.5);

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

	// float zone = sin(position.y * PI2 * 3 + PI/2);
	// float pressure = (zone);
	// float direction = abs(sin(position.y * PI2 + PI/2)) * 2 - 1;
	
	// float speed = direction * (pressure * 0.5 + 0.5);
	
	// if (mode == 3) {
	// 	output = (float3(-1.0,  -1.0, speed) * 0.5 + 0.5);
	// }

	// if (mode == 4) {
	// 	output = (float3(pressure, -1.0,  -1.0) * 0.5 + 0.5);
	// }
	// if (mode == 5) {

	// }

	// if (mode == 6) {
	// 	output = float3(position.xy, 1);
	// }
	// o.color = float4(hash(uint3(position.xy * 1000, 1.0)), 1.0);
	// o.color = float4(position, 1.0, 1.0);
	// o.color = float4(subregion.xyz, 1);
	

	o.color = float4(output, 1.0);
	return o;
}
