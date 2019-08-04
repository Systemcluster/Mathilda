#ifndef WNOISE3D
#define WNOISE3D

// from: https://github.com/Scrawk/GPU-Voronoi-Noise/blob/master/Assets/GPUVoronoiNoise/Shader/GPUVoronoiNoise4D.cginc

#include "noise/util.hlsl"

//1/7
#define K 0.142857142857
//3/7
#define Ko 0.428571428571

float2 wnoise3d(float3 P, float jitter)
{			
	float3 Pi = mod(floor(P), 289.0);
 	float3 Pf = frac(P);
	float3 oi = float3(-1.0, 0.0, 1.0);
	float3 of = float3(-0.5, 0.5, 1.5);
	float3 px = permute(Pi.x + oi);
	float3 py = permute(Pi.y + oi);

	float3 p, ox, oy, oz, dx, dy, dz;
	float2 F = 1e6;

	for(int i = 0; i < 3; i++)
	{
		for(int j = 0; j < 3; j++)
		{
			p = permute(px[i] + py[j] + Pi.z + oi); // pij1, pij2, pij3

			ox = frac(p*K) - Ko;
			oy = mod(floor(p*K),7.0)*K - Ko;
			
			p = permute(p);
			
			oz = frac(p*K) - Ko;
		
			dx = Pf.x - of[i] + jitter*ox;
			dy = Pf.y - of[j] + jitter*oy;
			dz = Pf.z - of + jitter*oz;
			
			float3 d = dx * dx + dy * dy + dz * dz; // dij1, dij2 and dij3, squared
			
			//Find lowest and second lowest distances
			for(int n = 0; n < 3; n++)
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
	
	return F;
}

#undef K
#undef Ko

#endif
