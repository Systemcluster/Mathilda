#pragma shader_stage(vertex)

cbuffer GlobalBuffer : register(b0) {
	float2 position;
	float2 size;
	float4 color;
};

struct Input {
	uint vertexID : SV_VERTEXID;
};

struct Output {
	float4 position : SV_POSITION;
	float4 uv : TEXCOORD0;
};

Output main(Input input) {
	Output o;
	float2 tl = float2(position.x-size.x, position.y+size.y);
	float2 bl = float2(position.x-size.x, position.y-size.y);
	float2 br = float2(position.x+size.x, position.y-size.y);
	float2 tr = float2(position.x+size.x, position.y+size.y);
	float2 positions[6] = {
		tl,
		bl,
		br,
		br,
		tr,
		tl,
	};
	o.position = float4(positions[input.vertexID], 0.0, 1.0);
	o.uv = float4(positions[input.vertexID], 1.0, 1.0);
	return o;
}
