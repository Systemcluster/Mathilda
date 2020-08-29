#pragma shader_stage(vertex)

static const float2 positions[6] = {
	float2(-1.0, 1.0),
	float2(-1.0, -1.0),
	float2(1.0, -1.0),

	float2(1.0, -1.0),
	float2(1.0, 1.0),
	float2(-1.0, 1.0),
};
// static const float2 positions[3] = {
// 	float2(0.0, -1.2),
// 	float2(1.2, 1.2),
// 	float2(-1.2, 1.2)
// };

struct Input {
	uint vertexID : SV_VERTEXID;
};

struct Output {
	float4 position : SV_POSITION;
	float4 uv : TEXCOORD0;
};

Output main(Input input) {
	Output o;
	o.position = float4(positions[input.vertexID], 0.0, 1.0);
	o.uv = float4(positions[input.vertexID], 1.0, 1.0);
	return o;
}
