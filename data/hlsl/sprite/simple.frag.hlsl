#pragma shader_stage(fragment)

cbuffer GlobalBuffer : register(b0) {
	float2 position;
	float2 size;
	float4 color;
};

struct Input {
	float4 fragCoord : SV_POSITION;
	float4 uv : TEXCOORD0;
};

struct Output {
	float4 color : SV_TARGET0;
};

Output main(Input input) {
	Output o;
	o.color = color;
	return o;
}
