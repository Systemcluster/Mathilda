#pragma shader_stage(vertex)

cbuffer Element : register(b0) {
	float3 position;
	float2 size;
	float4 color;
	float3 rotation;
};

cbuffer Camera : register(b1) {
	float4x4 projection;
}

struct Input {
	uint vertexID : SV_VERTEXID;
};

struct Output {
	float4 position : SV_POSITION;
	float4 uv : TEXCOORD0;
};

float2 rotate_point(float pointX, float pointY, float originX, float originY, float angle) {
    return float2(
        cos(angle) * (pointX-originX) - sin(angle) * (pointY-originY) + originX,
        sin(angle) * (pointX-originX) + cos(angle) * (pointY-originY) + originY
	);
}

Output main(Input input) {
	Output o;
	float a = rotation[0];

	// float r = float2x2(cos(a), -sin(a), sin(a), cos(a));
	// float3 tl = float3(float2(position.x-size.x, position.y+size.y) * r, position.z);
	// float3 bl = float3(float2(position.x-size.x, position.y-size.y) * r, position.z);
	// float3 br = float3(float2(position.x+size.x, position.y-size.y) * r, position.z);
	// float3 tr = float3(float2(position.x+size.x, position.y+size.y) * r, position.z);

	float3 tl = float3(rotate_point(-size.x, +size.y, 0, 0, a) + float2(position.x, position.y), position.z);
	float3 bl = float3(rotate_point(-size.x, -size.y, 0, 0, a) + float2(position.x, position.y), position.z);
	float3 br = float3(rotate_point(+size.x, -size.y, 0, 0, a) + float2(position.x, position.y), position.z);
	float3 tr = float3(rotate_point(+size.x, +size.y, 0, 0, a) + float2(position.x, position.y), position.z);

	float3 positions[6] = {
		tl,
		bl,
		br,
		br,
		tr,
		tl,
	};
	o.position = float4(positions[input.vertexID], 1.0) * projection;
	o.uv = float4(positions[input.vertexID], 1.0);
	return o;
}
