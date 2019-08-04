#ifndef UTIL
#define UTIL

#define mod(x, y) ((x) - (y)*floor((x) * (1.0 / (y))))
#define mod289(x) ((x)-floor((x) * (1.0 / 289.0)) * 289.0)
#define mod7(x) ((x)-floor((x) * (1.0 / 7.0)) * 7.0)

#define fade(t) ((t) * (t) * (t) * ((t) * ((t)*6.0 - 15.0) + 10.0))
// Permutation polynomial (ring size 289 = 17*17)
#define permute(x) (mod289((((x)*34.0) + 1.0) * (x)))

// Roughly matches rsqrt from 0.5 to 1
#define taylorInvSqrt(r) (1.79284291400159 - (r)*0.85373472095314)

#define inverse_smoothstep(x) ((0.5 - sin(asin(1.0 - 2.0 * clamp(x, 0, 1)) / 3.0)))

float3 hash(uint3 x) {
	static const uint k = 1664525U;
	x = ((x >> 8U) ^ x.yzx) * k;
	x = ((x >> 8U) ^ x.yzx) * k;
	x = ((x >> 8U) ^ x.yzx) * k;
	return float3(x) * (1.0 / float(0xffffffffU));
}


#endif
