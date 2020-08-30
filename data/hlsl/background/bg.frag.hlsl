#pragma shader_stage(fragment)

cbuffer Element : register(b2) {
	float3 position;
	float aspect;
};

Texture2D map : register(t0);
SamplerState sam : register(s1);

struct Input {
	float4 fragCoord : SV_POSITION;
	float2 uv : TEXCOORD0;
};

struct Output {
	float4 color : SV_TARGET0;
};

Output main(Input input) {
	Output o;
	float width;
	float height;
	map.GetDimensions(width, height);
	o.color = map.Sample(sam, float2(input.uv * 0.5 + 0.5));
	return o;
}
