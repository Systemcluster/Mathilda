#pragma shader_stage(fragment)

cbuffer Element : register(b0) {
	float3 position;
	float2 size;
	float4 color;
	float4 rotation;
	float2 texturecoords;
	float2 texturesize;
};

Texture2D map : register(t2);
SamplerState sam : register(s3);

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
	o.color = map.Sample(sam, float2(input.uv.x / width, input.uv.y / height));
	return o;
}
