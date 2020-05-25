#pragma shader_stage(compute)

cbuffer GlobalBuffer : register(b0) {
	float4 subregion;
	float2 offset;
	float time;
	int mode;
	float level;
};
RWTexture2D<float4> map : register(t1);

struct Input {
	uint3 id : SV_DispatchThreadID;
	uint3 Gid : SV_GroupID;
	uint3 GTid : SV_GroupThreadID;
	uint GI : SV_GroupIndex;
};

[numthreads(16, 16, 1)] void main(Input input) {
	uint2 dims;
	map.GetDimensions(dims.x, dims.y);



	// map[input.id.xy] *= 1.0;
}
