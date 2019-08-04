#ifndef WNOISE4D
#define WNOISE4D

// from: https://github.com/Scrawk/GPU-Voronoi-Noise/blob/master/Assets/GPUVoronoiNoise/Shader/GPUVoronoiNoise4D.cginc

#include "noise/util.hlsl"

//1/7
#define K 0.142857142857
//3/7
#define Ko 0.428571428571

float2 wnoise4d(float4 P, float jitter)
{			
	float4 Pi = mod(floor(P), 289.0);
 	float4 Pf = frac(P);
	float3 oi = float3(-1.0, 0.0, 1.0);
	float3 of = float3(-0.5, 0.5, 1.5);
	float3 px = permute(Pi.x + oi);
	float3 py = permute(Pi.y + oi);
	float3 pz = permute(Pi.z + oi);

	float3 p, ox, oy, oz, ow, dx, dy, dz, dw, d;
	float2 F = 1e6;
	int i, j, k, n;

	for(i = 0; i < 3; i++)
	{
		for(j = 0; j < 3; j++)
		{
			for(k = 0; k < 3; k++)
			{
				p = permute(px[i] + py[j] + pz[k] + Pi.w + oi); // pijk1, pijk2, pijk3
	
				ox = frac(p*K) - Ko;
				oy = mod(floor(p*K),7.0)*K - Ko;
				
				p = permute(p);
				
				oz = frac(p*K) - Ko;
				ow = mod(floor(p*K),7.0)*K - Ko;
			
				dx = Pf.x - of[i] + jitter*ox;
				dy = Pf.y - of[j] + jitter*oy;
				dz = Pf.z - of[k] + jitter*oz;
				dw = Pf.w - of + jitter*ow;
				
				d = dx * dx + dy * dy + dz * dz + dw * dw; // dijk1, dijk2 and dijk3, squared
				
				//Find the lowest and second lowest distances
				for(n = 0; n < 3; n++)
				{
					if(d[n] < F[0])
					{
						F[1] = F[0];
						F[0] = d[n];
					}
					else if(d[n] < F[1])
					{
						F[1] = d[n];
					}
				}
			}
		}
	}
	
	return F;
}

#undef K
#undef Ko

#endif
