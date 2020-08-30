#pragma shader_stage(vertex)

cbuffer Element : register(b2) {
	float3 position;
	float aspect;
};

struct Input {
	uint vertexID : SV_VERTEXID;
};

struct Output {
	float4 position : SV_POSITION;
	float2 uv : TEXCOORD0;
};

static const float2 positions[6] = {
	float2(-1.0, 1.0),
	float2(-1.0, -1.0),
	float2(1.0, -1.0),

	float2(1.0, -1.0),
	float2(1.0, 1.0),
	float2(-1.0, 1.0),
};

Output main(Input input) {
	Output o;
	o.position = float4(positions[input.vertexID], 1.0, 1.0);
	float2 uv = positions[input.vertexID] + position.xy / 10.0;
	o.uv = float4(uv.x * aspect, uv.y, 1.0, 1.0);
	return o;
}
