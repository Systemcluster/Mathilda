#ifndef WNOISE
#define WNOISE

// from: https://github.com/heyx3/GPUNoiseForUnity/blob/master/Assets/GPUGraph/GpuRand.cginc

#define _HASH(p4, swizzle)                                      \
	p4 = frac(p4 * float4(443.897, 441.423, 437.195, 444.129)); \
	p4 += dot(p4, p4.wzxy + 19.19);                             \
	return frac(dot(p.xyzw, p.zwxy) * p.swizzle);
float _hashTo1(float4 p) {
	_HASH(p, x);
}
float2 _hashTo2(float4 p) {
	_HASH(p, xy);
}
float3 _hashTo3(float4 p) {
	_HASH(p, xyz);
}
float4 _hashTo4(float4 p) {
	_HASH(p, xyzw);
}
#undef _HASH
float hashTo1(float p) { return _hashTo1(p.xxxx); }
float hashTo1(float2 p) { return _hashTo1(p.xyxy); }
float hashTo1(float3 p) { return _hashTo1(p.xyzx); }
float hashTo1(float4 p) { return _hashTo1(p); }
float2 hashTo2(float p) { return _hashTo2(p.xxxx); }
float2 hashTo2(float2 p) { return _hashTo2(p.xyxy); }
float2 hashTo2(float3 p) { return _hashTo2(p.xyzx); }
float2 hashTo2(float4 p) { return _hashTo2(p); }
float3 hashTo3(float p) { return _hashTo3(p.xxxx); }
float3 hashTo3(float2 p) { return _hashTo3(p.xyxy); }
float3 hashTo3(float3 p) { return _hashTo3(p.xyzx); }
float3 hashTo3(float4 p) { return _hashTo3(p); }
float4 hashTo4(float p) { return _hashTo4(p.xxxx); }
float4 hashTo4(float2 p) { return _hashTo4(p.xyxy); }
float4 hashTo4(float3 p) { return _hashTo4(p.xyzx); }
float4 hashTo4(float4 p) { return _hashTo4(p); }

#define IDENTITY(x) x
#define WRAP(x) (frac((x) / valMax) * valMax)

float wnoise(float2 f, float2 cellVariance) {
	const float3 zon = float3(0.0, 1.0, -1.0);
	float2 cellMidXY = floor(f),
		   cellMinXY = cellMidXY + zon.zz,
		   cellMidXMinY = cellMidXY + zon.xz,
		   cellMaxXMinY = cellMidXY + zon.yz,
		   cellMinXMidY = cellMidXY + zon.zx,
		   cellMaxXMidY = cellMidXY + zon.yx,
		   cellMinXMaxY = cellMidXY + zon.zy,
		   cellMidXMaxY = cellMidXY + zon.xy,
		   cellMaxXY = cellMidXY + zon.yy;
#define VAL(var) distance(f, var + lerp(0.5 - cellVariance, 0.5 + cellVariance, hashTo2(var)))
#define MIN3(a, b, c) min(a, min(b, c))
	return MIN3(MIN3(VAL(cellMinXY), VAL(cellMidXMinY), VAL(cellMaxXMinY)),
		MIN3(VAL(cellMinXMidY), VAL(cellMidXY), VAL(cellMaxXMidY)),
		MIN3(VAL(cellMinXMaxY), VAL(cellMidXMaxY), VAL(cellMaxXY)));
#undef VAL
#undef MIN3
}
float wnoise(float2 f, float2 cellVariance, float2 valMax) {
	const float3 zon = float3(0.0, 1.0, -1.0);
	float2 cellMidXY = floor(f),
		   cellMinXY = cellMidXY + zon.zz,
		   cellMidXMinY = cellMidXY + zon.xz,
		   cellMaxXMinY = cellMidXY + zon.yz,
		   cellMinXMidY = cellMidXY + zon.zx,
		   cellMaxXMidY = cellMidXY + zon.yx,
		   cellMinXMaxY = cellMidXY + zon.zy,
		   cellMidXMaxY = cellMidXY + zon.xy,
		   cellMaxXY = cellMidXY + zon.yy;
#define VAL(var) distance(f, var + lerp(0.5 - cellVariance, 0.5 + cellVariance, hashTo2(WRAP(var))))
#define MIN3(a, b, c) min(a, min(b, c))
	return MIN3(MIN3(VAL(cellMinXY), VAL(cellMidXMinY), VAL(cellMaxXMinY)),
		MIN3(VAL(cellMinXMidY), VAL(cellMidXY), VAL(cellMaxXMidY)),
		MIN3(VAL(cellMinXMaxY), VAL(cellMidXMaxY), VAL(cellMaxXY)));
#undef VAL
#undef MIN3
}

float wnoise(float3 f, float3 cellVariance) {
	float3 cellyyy = floor(f);

	const float3 c = float3(-1.0, 0.0, 1.0);
#define MAKE_VAL(swizzle) float3 cell##swizzle = cellyyy + c.swizzle;
	MAKE_VAL(xxx)
	MAKE_VAL(xxy)
	MAKE_VAL(xxz)
	MAKE_VAL(xyx)
	MAKE_VAL(xyy)
	MAKE_VAL(xyz)
	MAKE_VAL(xzx)
	MAKE_VAL(xzy)
	MAKE_VAL(xzz)
	MAKE_VAL(yxx)
	MAKE_VAL(yxy)
	MAKE_VAL(yxz)
	MAKE_VAL(yyx)
	MAKE_VAL(yyz)
	MAKE_VAL(yzx)
	MAKE_VAL(yzy)
	MAKE_VAL(yzz)
	MAKE_VAL(zxx)
	MAKE_VAL(zxy)
	MAKE_VAL(zxz)
	MAKE_VAL(zyx)
	MAKE_VAL(zyy)
	MAKE_VAL(zyz)
	MAKE_VAL(zzx)
	MAKE_VAL(zzy)
	MAKE_VAL(zzz)
#define VAL(swizzle) distance(f, cell##swizzle + lerp(0.5 - cellVariance, 0.5 + cellVariance, hashTo3(cell##swizzle)))
#define MIN3(a, b, c) min(a, min(b, c))
#define MIN9(a, b, c, d, e, f, g, h, i) MIN3(MIN3(a, b, c), MIN3(d, e, f), MIN3(g, h, i))
	return MIN3(MIN9(VAL(xxx), VAL(xxy), VAL(xxz),
					VAL(xyx), VAL(xyy), VAL(xyz),
					VAL(xzx), VAL(xzy), VAL(xzz)),
		MIN9(VAL(yxx), VAL(yxy), VAL(yxz),
			VAL(yyx), VAL(yyy), VAL(yyz),
			VAL(yzx), VAL(yzy), VAL(yzz)),
		MIN9(VAL(zxx), VAL(zxy), VAL(zxz),
			VAL(zyx), VAL(zyy), VAL(zyz),
			VAL(zzx), VAL(zzy), VAL(zzz)));
#undef MAKE_VAL
#undef VAL
#undef MIN3
#undef MIN9
}
float wnoise(float3 f, float3 cellVariance, float3 valMax) {
	float3 cellyyy = floor(f);

	const float3 c = float3(-1.0, 0.0, 1.0);
#define MAKE_VAL(swizzle) float3 cell##swizzle = cellyyy + c.swizzle;
	MAKE_VAL(xxx)
	MAKE_VAL(xxy)
	MAKE_VAL(xxz)
	MAKE_VAL(xyx)
	MAKE_VAL(xyy)
	MAKE_VAL(xyz)
	MAKE_VAL(xzx)
	MAKE_VAL(xzy)
	MAKE_VAL(xzz)
	MAKE_VAL(yxx)
	MAKE_VAL(yxy)
	MAKE_VAL(yxz)
	MAKE_VAL(yyx)
	MAKE_VAL(yyz)
	MAKE_VAL(yzx)
	MAKE_VAL(yzy)
	MAKE_VAL(yzz)
	MAKE_VAL(zxx)
	MAKE_VAL(zxy)
	MAKE_VAL(zxz)
	MAKE_VAL(zyx)
	MAKE_VAL(zyy)
	MAKE_VAL(zyz)
	MAKE_VAL(zzx)
	MAKE_VAL(zzy)
	MAKE_VAL(zzz)
#define VAL(swizzle) distance(f, cell##swizzle + lerp(0.5 - cellVariance, 0.5 + cellVariance, hashTo3(WRAP(cell##swizzle))))
#define MIN3(a, b, c) min(a, min(b, c))
#define MIN9(a, b, c, d, e, f, g, h, i) MIN3(MIN3(a, b, c), MIN3(d, e, f), MIN3(g, h, i))
	return MIN3(MIN9(VAL(xxx), VAL(xxy), VAL(xxz),
					VAL(xyx), VAL(xyy), VAL(xyz),
					VAL(xzx), VAL(xzy), VAL(xzz)),
		MIN9(VAL(yxx), VAL(yxy), VAL(yxz),
			VAL(yyx), VAL(yyy), VAL(yyz),
			VAL(yzx), VAL(yzy), VAL(yzz)),
		MIN9(VAL(zxx), VAL(zxy), VAL(zxz),
			VAL(zyx), VAL(zyy), VAL(zyz),
			VAL(zzx), VAL(zzy), VAL(zzz)));
#undef MAKE_VAL
#undef VAL
#undef MIN3
#undef MIN9
}

float wnoise(float4 f, float4 cellVariance) {
	float4 cellyyyy = floor(f);

	const float3 c = float3(-1.0, 0.0, 1.0);

	float4 cellPos;

	//Calculate the first noise value and store it.
	float4 cellOffsetMin = 0.5 - cellVariance,
		   cellOffsetMax = 0.5 + cellVariance;
#define GET distance(f, cellPos + lerp(cellOffsetMin, cellOffsetMax, hashTo4(cellPos)))
	cellPos = cellyyyy;
	float minNoise = GET;

	//Do the rest of the noise values.
	//Define a way to easily iterate over every possible swizzle.
#define DO(swizzle)                 \
	cellPos = cellyyyy + c.swizzle; \
	minNoise = min(minNoise, GET);
#define FOREACH_X(swizzleX) \
	FOREACH_XY(swizzleX##x) \
	FOREACH_XY(swizzleX##y) FOREACH_XY(swizzleX##z)
#define FOREACH_XY(swizzleXY) \
	FOREACH_XYZ(swizzleXY##x) \
	FOREACH_XYZ(swizzleXY##y) FOREACH_XYZ(swizzleXY##z)
#define FOREACH_XYZ(swizzleXYZ) \
	DO(swizzleXYZ##x)           \
	DO(swizzleXYZ##y) DO(swizzleXYZ##z)
	//Skip yyyy because we already did that one.
#define FOREACH_DO   \
	FOREACH_X(x)     \
	FOREACH_XY(yx)   \
	FOREACH_XYZ(yyx) \
	DO(yyyz)         \
	FOREACH_XYZ(yyz) \
	FOREACH_XY(yz)   \
	FOREACH_X(z)

	//Perform the iteration.
	FOREACH_DO;
	return minNoise;

#undef FOREACH_XYZ
#undef FOREACH_XY
#undef FOREACH_X
#undef FOREACH_DO
#undef DO
#undef GET
}
float wnoise(float4 f, float4 cellVariance, float4 valMax) {
	float4 cellyyyy = floor(f);

	const float3 c = float3(-1.0, 0.0, 1.0);

	float4 cellPos;

	//Calculate the first noise value and store it.
	float4 cellOffsetMin = 0.5 - cellVariance,
		   cellOffsetMax = 0.5 + cellVariance;
#define GET distance(f, cellPos + lerp(cellOffsetMin, cellOffsetMax, hashTo4(WRAP(cellPos))))
	cellPos = cellyyyy;
	float minNoise = GET;

	//Do the rest of the noise values.
	//Define a way to easily iterate over every possible swizzle.
#define DO(swizzle)                 \
	cellPos = cellyyyy + c.swizzle; \
	minNoise = min(minNoise, GET);
#define FOREACH_X(swizzleX) \
	FOREACH_XY(swizzleX##x) \
	FOREACH_XY(swizzleX##y) FOREACH_XY(swizzleX##z)
#define FOREACH_XY(swizzleXY) \
	FOREACH_XYZ(swizzleXY##x) \
	FOREACH_XYZ(swizzleXY##y) FOREACH_XYZ(swizzleXY##z)
#define FOREACH_XYZ(swizzleXYZ) \
	DO(swizzleXYZ##x)           \
	DO(swizzleXYZ##y) DO(swizzleXYZ##z)
//Skip yyyy because we already did that one.
#define FOREACH_DO         \
	FOREACH_X(x)           \
	FOREACH_XY(yx)         \
	FOREACH_XYZ(yyx)       \
	DO(yyyx)               \
	DO(yyyz)               \
		FOREACH_XYZ(yyz)   \
			FOREACH_XY(yz) \
				FOREACH_X(z)

	//Perform the iteration.
	FOREACH_DO;
	return minNoise;

#undef FOREACH_XYZ
#undef FOREACH_XY
#undef FOREACH_X
#undef FOREACH_DO
#undef DO
#undef GET
}

#undef IDENTITY
#undef WRAP

#endif
