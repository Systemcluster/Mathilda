struct Input {
	float4 fragCoord : SV_POSITION;
	float4 uv : TEXCOORD0;
};

struct Output {
	float4 color : SV_TARGET0;
};

Output main(Input input) {
	Output o;

	return o;
}
