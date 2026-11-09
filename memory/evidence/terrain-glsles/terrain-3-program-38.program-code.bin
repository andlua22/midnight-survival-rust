#ifdef VERTEX
#version 300 es

#define HLSLCC_ENABLE_UNIFORM_BUFFERS 1
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
#define UNITY_UNIFORM
#else
#define UNITY_UNIFORM uniform
#endif
#define UNITY_SUPPORTS_UNIFORM_LOCATION 1
#if UNITY_SUPPORTS_UNIFORM_LOCATION
#define UNITY_LOCATION(x) layout(location = x)
#define UNITY_BINDING(x) layout(binding = x, std140)
#else
#define UNITY_LOCATION(x)
#define UNITY_BINDING(x) layout(std140)
#endif
uniform 	vec3 _WorldSpaceCameraPos;
uniform 	vec4 _ProjectionParams;
uniform 	vec4 unity_FogParams;
uniform 	vec4 hlslcc_mtx4x4unity_MatrixVP[4];
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
UNITY_BINDING(0) uniform UnityPerDraw {
#endif
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_ObjectToWorld[4];
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_WorldToObject[4];
	UNITY_UNIFORM vec4 unity_LODFade;
	UNITY_UNIFORM mediump vec4 unity_WorldTransformParams;
	UNITY_UNIFORM vec4 unity_RenderingLayer;
	UNITY_UNIFORM mediump vec4 unity_LightData;
	UNITY_UNIFORM mediump vec4 unity_LightIndices[2];
	UNITY_UNIFORM vec4 unity_ProbesOcclusion;
	UNITY_UNIFORM mediump vec4 unity_SpecCube0_HDR;
	UNITY_UNIFORM mediump vec4 unity_SpecCube1_HDR;
	UNITY_UNIFORM vec4 unity_SpecCube0_BoxMax;
	UNITY_UNIFORM vec4 unity_SpecCube0_BoxMin;
	UNITY_UNIFORM vec4 unity_SpecCube0_ProbePosition;
	UNITY_UNIFORM vec4 unity_SpecCube1_BoxMax;
	UNITY_UNIFORM vec4 unity_SpecCube1_BoxMin;
	UNITY_UNIFORM vec4 unity_SpecCube1_ProbePosition;
	UNITY_UNIFORM vec4 unity_LightmapST;
	UNITY_UNIFORM vec4 unity_DynamicLightmapST;
	UNITY_UNIFORM mediump vec4 unity_SHAr;
	UNITY_UNIFORM mediump vec4 unity_SHAg;
	UNITY_UNIFORM mediump vec4 unity_SHAb;
	UNITY_UNIFORM mediump vec4 unity_SHBr;
	UNITY_UNIFORM mediump vec4 unity_SHBg;
	UNITY_UNIFORM mediump vec4 unity_SHBb;
	UNITY_UNIFORM mediump vec4 unity_SHC;
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_MatrixPreviousM[4];
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_MatrixPreviousMI[4];
	UNITY_UNIFORM vec4 unity_MotionVectorsParams;
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
};
#endif
in highp vec4 in_POSITION0;
in highp vec3 in_NORMAL0;
in highp vec2 in_TEXCOORD0;
out highp vec2 vs_TEXCOORD0;
out mediump float vs_TEXCOORD5;
out highp vec3 vs_TEXCOORD1;
out highp vec3 vs_TEXCOORD3;
out highp vec3 vs_TEXCOORD4;
out highp vec3 vs_TEXCOORD7;
out mediump vec3 vs_TEXCOORD8;
out highp vec4 vs_TEXCOORD11;
vec4 u_xlat0;
vec4 u_xlat1;
vec4 u_xlat2;
mediump vec4 u_xlat16_2;
mediump vec3 u_xlat16_3;
mediump vec3 u_xlat16_4;
float u_xlat7;
float u_xlat15;
void main()
{
    vs_TEXCOORD0.xy = in_TEXCOORD0.xy;
    u_xlat0.xyz = in_POSITION0.yyy * hlslcc_mtx4x4unity_ObjectToWorld[1].xyz;
    u_xlat0.xyz = hlslcc_mtx4x4unity_ObjectToWorld[0].xyz * in_POSITION0.xxx + u_xlat0.xyz;
    u_xlat0.xyz = hlslcc_mtx4x4unity_ObjectToWorld[2].xyz * in_POSITION0.zzz + u_xlat0.xyz;
    u_xlat0.xyz = u_xlat0.xyz + hlslcc_mtx4x4unity_ObjectToWorld[3].xyz;
    vs_TEXCOORD1.xyz = u_xlat0.xyz;
    u_xlat1.xyz = in_NORMAL0.yyy * hlslcc_mtx4x4unity_ObjectToWorld[1].xyz;
    u_xlat1.xyz = hlslcc_mtx4x4unity_ObjectToWorld[0].xyz * in_NORMAL0.xxx + u_xlat1.xyz;
    u_xlat1.xyz = hlslcc_mtx4x4unity_ObjectToWorld[2].xyz * in_NORMAL0.zzz + u_xlat1.xyz;
    u_xlat15 = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlat15 = max(u_xlat15, 1.17549435e-38);
    u_xlat15 = inversesqrt(u_xlat15);
    u_xlat1.xyz = vec3(u_xlat15) * u_xlat1.xyz;
    u_xlat15 = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlat15 = inversesqrt(u_xlat15);
    u_xlat1.xyz = vec3(u_xlat15) * u_xlat1.xyz;
    vs_TEXCOORD3.xyz = u_xlat1.xyz;
    vs_TEXCOORD4.xyz = (-u_xlat0.xyz) + _WorldSpaceCameraPos.xyz;
    u_xlat2 = u_xlat0.yyyy * hlslcc_mtx4x4unity_MatrixVP[1];
    u_xlat2 = hlslcc_mtx4x4unity_MatrixVP[0] * u_xlat0.xxxx + u_xlat2;
    u_xlat0 = hlslcc_mtx4x4unity_MatrixVP[2] * u_xlat0.zzzz + u_xlat2;
    u_xlat0 = u_xlat0 + hlslcc_mtx4x4unity_MatrixVP[3];
    u_xlat2.x = u_xlat0.z + _ProjectionParams.y;
    u_xlat7 = _ProjectionParams.y + _ProjectionParams.z;
    u_xlat2.x = u_xlat2.x / u_xlat7;
    u_xlat2.x = u_xlat2.x * _ProjectionParams.z;
    u_xlat2.x = max(u_xlat2.x, 0.0);
    u_xlat2.x = u_xlat2.x * unity_FogParams.z + unity_FogParams.w;
    u_xlat2.x = clamp(u_xlat2.x, 0.0, 1.0);
    vs_TEXCOORD5 = u_xlat2.x;
    vs_TEXCOORD7.xyz = vec3(0.0, 0.0, 0.0);
    u_xlat16_3.x = u_xlat1.y * u_xlat1.y;
    u_xlat16_3.x = u_xlat1.x * u_xlat1.x + (-u_xlat16_3.x);
    u_xlat16_2 = u_xlat1.yzzx * u_xlat1.xyzz;
    u_xlat16_4.x = dot(unity_SHBr, u_xlat16_2);
    u_xlat16_4.y = dot(unity_SHBg, u_xlat16_2);
    u_xlat16_4.z = dot(unity_SHBb, u_xlat16_2);
    u_xlat16_3.xyz = unity_SHC.xyz * u_xlat16_3.xxx + u_xlat16_4.xyz;
    u_xlat1.w = 1.0;
    u_xlat16_4.x = dot(unity_SHAr, u_xlat1);
    u_xlat16_4.y = dot(unity_SHAg, u_xlat1);
    u_xlat16_4.z = dot(unity_SHAb, u_xlat1);
    u_xlat16_3.xyz = u_xlat16_3.xyz + u_xlat16_4.xyz;
    vs_TEXCOORD8.xyz = max(u_xlat16_3.xyz, vec3(0.0, 0.0, 0.0));
    gl_Position = u_xlat0;
    u_xlat0.y = u_xlat0.y * _ProjectionParams.x;
    u_xlat1.xzw = u_xlat0.xwy * vec3(0.5, 0.5, 0.5);
    vs_TEXCOORD11.zw = u_xlat0.zw;
    vs_TEXCOORD11.xy = u_xlat1.zz + u_xlat1.xw;
    return;
}

#endif
#ifdef FRAGMENT
#version 300 es
#ifdef GL_EXT_shader_texture_lod
#extension GL_EXT_shader_texture_lod : enable
#endif

precision highp float;
precision highp int;
#define HLSLCC_ENABLE_UNIFORM_BUFFERS 1
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
#define UNITY_UNIFORM
#else
#define UNITY_UNIFORM uniform
#endif
#define UNITY_SUPPORTS_UNIFORM_LOCATION 1
#if UNITY_SUPPORTS_UNIFORM_LOCATION
#define UNITY_LOCATION(x) layout(location = x)
#define UNITY_BINDING(x) layout(binding = x, std140)
#else
#define UNITY_LOCATION(x)
#define UNITY_BINDING(x) layout(std140)
#endif
uniform 	vec4 _MainLightPosition;
uniform 	mediump vec4 _MainLightColor;
uniform 	vec4 _ProjectionParams;
uniform 	vec2 _GlobalMipBias;
uniform 	vec4 unity_FogParams;
uniform 	mediump vec4 unity_FogColor;
uniform 	vec4 hlslcc_mtx4x4unity_MatrixV[4];
uniform 	vec4 hlslcc_mtx4x4_MainLightWorldToShadow[20];
uniform 	vec4 _CascadeShadowSplitSpheres0;
uniform 	vec4 _CascadeShadowSplitSpheres1;
uniform 	vec4 _CascadeShadowSplitSpheres2;
uniform 	vec4 _CascadeShadowSplitSpheres3;
uniform 	vec4 _CascadeShadowSplitSphereRadii;
uniform 	mediump vec4 _MainLightShadowParams;
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
UNITY_BINDING(0) uniform UnityPerDraw {
#endif
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_ObjectToWorld[4];
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_WorldToObject[4];
	UNITY_UNIFORM vec4 unity_LODFade;
	UNITY_UNIFORM mediump vec4 unity_WorldTransformParams;
	UNITY_UNIFORM vec4 unity_RenderingLayer;
	UNITY_UNIFORM mediump vec4 unity_LightData;
	UNITY_UNIFORM mediump vec4 unity_LightIndices[2];
	UNITY_UNIFORM vec4 unity_ProbesOcclusion;
	UNITY_UNIFORM mediump vec4 unity_SpecCube0_HDR;
	UNITY_UNIFORM mediump vec4 unity_SpecCube1_HDR;
	UNITY_UNIFORM vec4 unity_SpecCube0_BoxMax;
	UNITY_UNIFORM vec4 unity_SpecCube0_BoxMin;
	UNITY_UNIFORM vec4 unity_SpecCube0_ProbePosition;
	UNITY_UNIFORM vec4 unity_SpecCube1_BoxMax;
	UNITY_UNIFORM vec4 unity_SpecCube1_BoxMin;
	UNITY_UNIFORM vec4 unity_SpecCube1_ProbePosition;
	UNITY_UNIFORM vec4 unity_LightmapST;
	UNITY_UNIFORM vec4 unity_DynamicLightmapST;
	UNITY_UNIFORM mediump vec4 unity_SHAr;
	UNITY_UNIFORM mediump vec4 unity_SHAg;
	UNITY_UNIFORM mediump vec4 unity_SHAb;
	UNITY_UNIFORM mediump vec4 unity_SHBr;
	UNITY_UNIFORM mediump vec4 unity_SHBg;
	UNITY_UNIFORM mediump vec4 unity_SHBb;
	UNITY_UNIFORM mediump vec4 unity_SHC;
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_MatrixPreviousM[4];
	UNITY_UNIFORM vec4 hlslcc_mtx4x4unity_MatrixPreviousMI[4];
	UNITY_UNIFORM vec4 unity_MotionVectorsParams;
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
};
#endif
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
UNITY_BINDING(1) uniform UnityPerMaterial {
#endif
	UNITY_UNIFORM vec4 _Control_ST;
	UNITY_UNIFORM mediump float _Splat0_Smoothness;
	UNITY_UNIFORM mediump float _Splat1_Smoothness;
	UNITY_UNIFORM mediump float _Splat0_S;
	UNITY_UNIFORM mediump float _Splat1_S;
	UNITY_UNIFORM mediump float _Splat2_Smoothness;
	UNITY_UNIFORM mediump float _Splat2_S;
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
};
#endif
UNITY_LOCATION(0) uniform mediump sampler2D _Control;
UNITY_LOCATION(1) uniform mediump sampler2D _Splat0;
UNITY_LOCATION(2) uniform mediump sampler2D _Splat1;
UNITY_LOCATION(3) uniform mediump sampler2D _Splat2;
UNITY_LOCATION(4) uniform mediump sampler2D _MainLightShadowmapTexture;
UNITY_LOCATION(5) uniform mediump sampler2DShadow hlslcc_zcmp_MainLightShadowmapTexture;
in highp vec2 vs_TEXCOORD0;
in highp vec3 vs_TEXCOORD1;
in highp vec3 vs_TEXCOORD3;
in mediump vec3 vs_TEXCOORD8;
layout(location = 0) out mediump vec4 SV_TARGET0;
vec4 u_xlat0;
mediump vec4 u_xlat16_0;
bvec4 u_xlatb0;
vec3 u_xlat1;
mediump vec3 u_xlat16_1;
int u_xlati1;
uint u_xlatu1;
mediump vec3 u_xlat16_2;
vec3 u_xlat3;
mediump vec3 u_xlat16_3;
bool u_xlatb3;
mediump vec3 u_xlat16_4;
mediump vec3 u_xlat16_5;
mediump vec3 u_xlat16_6;
vec3 u_xlat8;
bool u_xlatb8;
float u_xlat22;
bool u_xlatb22;
mediump float u_xlat16_23;
void main()
{
    u_xlat0.xyz = vs_TEXCOORD1.xyz + (-_CascadeShadowSplitSpheres0.xyz);
    u_xlat0.x = dot(u_xlat0.xyz, u_xlat0.xyz);
    u_xlat1.xyz = vs_TEXCOORD1.xyz + (-_CascadeShadowSplitSpheres1.xyz);
    u_xlat0.y = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlat1.xyz = vs_TEXCOORD1.xyz + (-_CascadeShadowSplitSpheres2.xyz);
    u_xlat0.z = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlat1.xyz = vs_TEXCOORD1.xyz + (-_CascadeShadowSplitSpheres3.xyz);
    u_xlat0.w = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlatb0 = lessThan(u_xlat0, _CascadeShadowSplitSphereRadii);
    u_xlat16_2.x = (u_xlatb0.x) ? float(-1.0) : float(-0.0);
    u_xlat16_2.y = (u_xlatb0.y) ? float(-1.0) : float(-0.0);
    u_xlat16_2.z = (u_xlatb0.z) ? float(-1.0) : float(-0.0);
    u_xlat16_0.x = (u_xlatb0.x) ? float(1.0) : float(0.0);
    u_xlat16_0.y = (u_xlatb0.y) ? float(1.0) : float(0.0);
    u_xlat16_0.z = (u_xlatb0.z) ? float(1.0) : float(0.0);
    u_xlat16_0.w = (u_xlatb0.w) ? float(1.0) : float(0.0);
    u_xlat16_2.xyz = u_xlat16_2.xyz + u_xlat16_0.yzw;
    u_xlat16_0.yzw = max(u_xlat16_2.xyz, vec3(0.0, 0.0, 0.0));
    u_xlat16_2.x = dot(u_xlat16_0, vec4(4.0, 3.0, 2.0, 1.0));
    u_xlat16_2.x = (-u_xlat16_2.x) + 4.0;
    u_xlatu1 = uint(u_xlat16_2.x);
    u_xlati1 = int(int(u_xlatu1) << 2);
    u_xlat8.xyz = vs_TEXCOORD1.yyy * hlslcc_mtx4x4_MainLightWorldToShadow[(u_xlati1 + 1)].xyz;
    u_xlat8.xyz = hlslcc_mtx4x4_MainLightWorldToShadow[u_xlati1].xyz * vs_TEXCOORD1.xxx + u_xlat8.xyz;
    u_xlat8.xyz = hlslcc_mtx4x4_MainLightWorldToShadow[(u_xlati1 + 2)].xyz * vs_TEXCOORD1.zzz + u_xlat8.xyz;
    u_xlat1.xyz = u_xlat8.xyz + hlslcc_mtx4x4_MainLightWorldToShadow[(u_xlati1 + 3)].xyz;
    u_xlatb22 = 0.0>=u_xlat1.z;
    u_xlatb3 = u_xlat1.z>=1.0;
    vec3 txVec0 = vec3(u_xlat1.xy,u_xlat1.z);
    u_xlat16_1.x = textureLod(hlslcc_zcmp_MainLightShadowmapTexture, txVec0, 0.0);
    u_xlatb8 = u_xlatb22 || u_xlatb3;
    u_xlat16_2.x = (-_MainLightShadowParams.x) + 1.0;
    u_xlat16_2.x = u_xlat16_1.x * _MainLightShadowParams.x + u_xlat16_2.x;
    u_xlat16_2.x = (u_xlatb8) ? 1.0 : u_xlat16_2.x;
    u_xlat1.x = u_xlat16_2.x * unity_LightData.z;
    u_xlat1.xyz = u_xlat1.xxx * _MainLightColor.xyz;
    u_xlat22 = dot(vs_TEXCOORD3.xyz, vs_TEXCOORD3.xyz);
    u_xlat22 = inversesqrt(u_xlat22);
    u_xlat3.xyz = vec3(u_xlat22) * vs_TEXCOORD3.xyz;
    u_xlat16_2.x = dot(u_xlat3.xyz, _MainLightPosition.xyz);
    u_xlat16_2.x = clamp(u_xlat16_2.x, 0.0, 1.0);
    u_xlat16_2.xyz = u_xlat1.xyz * u_xlat16_2.xxx + vs_TEXCOORD8.xyz;
    u_xlat1.xy = vs_TEXCOORD0.xy * vec2(vec2(_Splat2_S, _Splat2_S));
    u_xlat16_1.xyz = texture(_Splat2, u_xlat1.xy, _GlobalMipBias.x).xyz;
    u_xlat0 = vs_TEXCOORD0.xyxy * vec4(_Splat0_S, _Splat0_S, _Splat1_S, _Splat1_S);
    u_xlat16_3.xyz = texture(_Splat0, u_xlat0.xy, _GlobalMipBias.x).xyz;
    u_xlat16_4.xyz = texture(_Splat1, u_xlat0.zw, _GlobalMipBias.x).xyz;
    u_xlat16_0 = texture(_Control, vs_TEXCOORD0.xy, _GlobalMipBias.x);
    u_xlat16_5.xyz = u_xlat16_0.yyy * u_xlat16_4.xyz;
    u_xlat16_5.xyz = u_xlat16_3.xyz * u_xlat16_0.xxx + u_xlat16_5.xyz;
    u_xlat16_5.xyz = u_xlat16_1.xyz * u_xlat16_0.zzz + u_xlat16_5.xyz;
    u_xlat16_6.xyz = u_xlat16_3.xyz + (-u_xlat16_5.xyz);
    u_xlat16_23 = (-u_xlat16_0.x) + 1.0;
    u_xlat16_23 = (-u_xlat16_0.y) + u_xlat16_23;
    u_xlat16_23 = (-u_xlat16_0.z) + u_xlat16_23;
    u_xlat16_23 = (-u_xlat16_0.w) + u_xlat16_23;
    u_xlat16_23 = clamp(u_xlat16_23, 0.0, 1.0);
    u_xlat16_5.xyz = vec3(u_xlat16_23) * u_xlat16_6.xyz + u_xlat16_5.xyz;
    u_xlat16_2.xyz = u_xlat16_2.xyz * u_xlat16_5.xyz + (-unity_FogColor.xyz);
    u_xlat1.x = vs_TEXCOORD1.y * hlslcc_mtx4x4unity_MatrixV[1].z;
    u_xlat1.x = hlslcc_mtx4x4unity_MatrixV[0].z * vs_TEXCOORD1.x + u_xlat1.x;
    u_xlat1.x = hlslcc_mtx4x4unity_MatrixV[2].z * vs_TEXCOORD1.z + u_xlat1.x;
    u_xlat1.x = u_xlat1.x + hlslcc_mtx4x4unity_MatrixV[3].z;
    u_xlat1.x = (-u_xlat1.x) + (-_ProjectionParams.y);
    u_xlat1.x = max(u_xlat1.x, 0.0);
    u_xlat1.x = u_xlat1.x * unity_FogParams.z + unity_FogParams.w;
    u_xlat1.x = clamp(u_xlat1.x, 0.0, 1.0);
    SV_TARGET0.xyz = u_xlat1.xxx * u_xlat16_2.xyz + unity_FogColor.xyz;
    SV_TARGET0.w = 1.0;
    return;
}

#endif
