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
mediump vec3 u_xlat16_2;
mediump vec4 u_xlat16_3;
mediump vec3 u_xlat16_4;
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
    vs_TEXCOORD5 = 0.0;
    vs_TEXCOORD7.xyz = vec3(0.0, 0.0, 0.0);
    u_xlat16_2.x = u_xlat1.y * u_xlat1.y;
    u_xlat16_2.x = u_xlat1.x * u_xlat1.x + (-u_xlat16_2.x);
    u_xlat16_3 = u_xlat1.yzzx * u_xlat1.xyzz;
    u_xlat16_4.x = dot(unity_SHBr, u_xlat16_3);
    u_xlat16_4.y = dot(unity_SHBg, u_xlat16_3);
    u_xlat16_4.z = dot(unity_SHBb, u_xlat16_3);
    u_xlat16_2.xyz = unity_SHC.xyz * u_xlat16_2.xxx + u_xlat16_4.xyz;
    u_xlat1.w = 1.0;
    u_xlat16_3.x = dot(unity_SHAr, u_xlat1);
    u_xlat16_3.y = dot(unity_SHAg, u_xlat1);
    u_xlat16_3.z = dot(unity_SHAb, u_xlat1);
    u_xlat16_2.xyz = u_xlat16_2.xyz + u_xlat16_3.xyz;
    vs_TEXCOORD8.xyz = max(u_xlat16_2.xyz, vec3(0.0, 0.0, 0.0));
    u_xlat1 = u_xlat0.yyyy * hlslcc_mtx4x4unity_MatrixVP[1];
    u_xlat1 = hlslcc_mtx4x4unity_MatrixVP[0] * u_xlat0.xxxx + u_xlat1;
    u_xlat0 = hlslcc_mtx4x4unity_MatrixVP[2] * u_xlat0.zzzz + u_xlat1;
    u_xlat0 = u_xlat0 + hlslcc_mtx4x4unity_MatrixVP[3];
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
uniform 	vec3 _WorldSpaceCameraPos;
uniform 	vec2 _GlobalMipBias;
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
	UNITY_UNIFORM mediump float _Splat3_Smoothness;
	UNITY_UNIFORM mediump float _Splat3_S;
#if HLSLCC_ENABLE_UNIFORM_BUFFERS
};
#endif
UNITY_LOCATION(0) uniform mediump samplerCube unity_SpecCube0;
UNITY_LOCATION(1) uniform mediump sampler2D _Control;
UNITY_LOCATION(2) uniform mediump sampler2D _Splat0;
UNITY_LOCATION(3) uniform mediump sampler2D _Splat1;
UNITY_LOCATION(4) uniform mediump sampler2D _Splat2;
UNITY_LOCATION(5) uniform mediump sampler2D _Splat3;
UNITY_LOCATION(6) uniform mediump sampler2D _MainLightShadowmapTexture;
UNITY_LOCATION(7) uniform mediump sampler2DShadow hlslcc_zcmp_MainLightShadowmapTexture;
in highp vec2 vs_TEXCOORD0;
in highp vec3 vs_TEXCOORD1;
in highp vec3 vs_TEXCOORD3;
in highp vec3 vs_TEXCOORD4;
in mediump vec3 vs_TEXCOORD8;
layout(location = 0) out mediump vec4 SV_TARGET0;
vec4 u_xlat0;
mediump vec4 u_xlat16_0;
bvec4 u_xlatb0;
vec3 u_xlat1;
mediump float u_xlat16_1;
int u_xlati1;
uint u_xlatu1;
mediump vec3 u_xlat16_2;
vec3 u_xlat3;
mediump vec4 u_xlat16_3;
bool u_xlatb3;
vec3 u_xlat4;
mediump vec4 u_xlat16_5;
vec2 u_xlat6;
mediump vec4 u_xlat16_6;
vec4 u_xlat7;
mediump vec4 u_xlat16_7;
mediump vec4 u_xlat16_8;
mediump vec4 u_xlat16_9;
mediump vec2 u_xlat16_10;
mediump vec3 u_xlat16_11;
mediump vec3 u_xlat16_12;
vec3 u_xlat14;
bool u_xlatb14;
mediump float u_xlat16_15;
mediump vec3 u_xlat16_23;
mediump vec3 u_xlat16_24;
bool u_xlatb40;
mediump float u_xlat16_41;
mediump float u_xlat16_44;
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
    u_xlat14.xyz = vs_TEXCOORD1.yyy * hlslcc_mtx4x4_MainLightWorldToShadow[(u_xlati1 + 1)].xyz;
    u_xlat14.xyz = hlslcc_mtx4x4_MainLightWorldToShadow[u_xlati1].xyz * vs_TEXCOORD1.xxx + u_xlat14.xyz;
    u_xlat14.xyz = hlslcc_mtx4x4_MainLightWorldToShadow[(u_xlati1 + 2)].xyz * vs_TEXCOORD1.zzz + u_xlat14.xyz;
    u_xlat1.xyz = u_xlat14.xyz + hlslcc_mtx4x4_MainLightWorldToShadow[(u_xlati1 + 3)].xyz;
    u_xlatb40 = 0.0>=u_xlat1.z;
    u_xlatb3 = u_xlat1.z>=1.0;
    vec3 txVec0 = vec3(u_xlat1.xy,u_xlat1.z);
    u_xlat16_1 = textureLod(hlslcc_zcmp_MainLightShadowmapTexture, txVec0, 0.0);
    u_xlatb14 = u_xlatb40 || u_xlatb3;
    u_xlat16_2.x = (-_MainLightShadowParams.x) + 1.0;
    u_xlat16_2.x = u_xlat16_1 * _MainLightShadowParams.x + u_xlat16_2.x;
    u_xlat16_2.x = (u_xlatb14) ? 1.0 : u_xlat16_2.x;
    u_xlat16_15 = (-u_xlat16_2.x) + 1.0;
    u_xlat1.xyz = vs_TEXCOORD1.xyz + (-_WorldSpaceCameraPos.xyz);
    u_xlat1.x = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlat1.x = u_xlat1.x * _MainLightShadowParams.z + _MainLightShadowParams.w;
    u_xlat1.x = clamp(u_xlat1.x, 0.0, 1.0);
    u_xlat16_2.x = u_xlat1.x * u_xlat16_15 + u_xlat16_2.x;
    u_xlat1.x = u_xlat16_2.x * unity_LightData.z;
    u_xlat14.x = dot(vs_TEXCOORD3.xyz, vs_TEXCOORD3.xyz);
    u_xlat14.x = inversesqrt(u_xlat14.x);
    u_xlat14.xyz = u_xlat14.xxx * vs_TEXCOORD3.xyz;
    u_xlat16_2.x = dot(u_xlat14.xyz, _MainLightPosition.xyz);
    u_xlat16_2.x = clamp(u_xlat16_2.x, 0.0, 1.0);
    u_xlat16_2.x = u_xlat1.x * u_xlat16_2.x;
    u_xlat16_2.xyz = u_xlat16_2.xxx * _MainLightColor.xyz;
    u_xlat1.x = dot(vs_TEXCOORD4.xyz, vs_TEXCOORD4.xyz);
    u_xlat1.x = max(u_xlat1.x, 1.17549435e-38);
    u_xlat1.x = inversesqrt(u_xlat1.x);
    u_xlat3.xyz = u_xlat1.xxx * vs_TEXCOORD4.xyz;
    u_xlat4.xyz = vs_TEXCOORD4.xyz * u_xlat1.xxx + _MainLightPosition.xyz;
    u_xlat16_41 = dot((-u_xlat3.xyz), u_xlat14.xyz);
    u_xlat16_41 = u_xlat16_41 + u_xlat16_41;
    u_xlat16_5.xyz = u_xlat14.xyz * (-vec3(u_xlat16_41)) + (-u_xlat3.xyz);
    u_xlat16_41 = dot(u_xlat14.xyz, u_xlat3.xyz);
    u_xlat16_41 = clamp(u_xlat16_41, 0.0, 1.0);
    u_xlat16_41 = (-u_xlat16_41) + 1.0;
    u_xlat16_41 = u_xlat16_41 * u_xlat16_41;
    u_xlat16_41 = u_xlat16_41 * u_xlat16_41;
    u_xlat0 = vs_TEXCOORD0.xyxy * vec4(_Splat0_S, _Splat0_S, _Splat1_S, _Splat1_S);
    u_xlat16_3 = texture(_Splat1, u_xlat0.zw, _GlobalMipBias.x);
    u_xlat16_0 = texture(_Splat0, u_xlat0.xy, _GlobalMipBias.x);
    u_xlat6.y = u_xlat16_3.w;
    u_xlat6.x = u_xlat16_0.w;
    u_xlat16_6.xy = u_xlat6.xy * vec2(_Splat0_Smoothness, _Splat1_Smoothness);
    u_xlat7 = vs_TEXCOORD0.xyxy * vec4(_Splat2_S, _Splat2_S, _Splat3_S, _Splat3_S);
    u_xlat16_8 = texture(_Splat2, u_xlat7.xy, _GlobalMipBias.x);
    u_xlat16_7 = texture(_Splat3, u_xlat7.zw, _GlobalMipBias.x);
    u_xlat16_6.z = u_xlat16_8.w * _Splat2_Smoothness;
    u_xlat16_6.w = u_xlat16_7.w * _Splat3_Smoothness;
    u_xlat16_9 = texture(_Control, vs_TEXCOORD0.xy, _GlobalMipBias.x);
    u_xlat16_44 = dot(u_xlat16_9, u_xlat16_6);
    u_xlat16_10.x = (-u_xlat16_44) + 1.0;
    u_xlat16_44 = u_xlat16_44 + 0.0400000215;
    u_xlat16_44 = clamp(u_xlat16_44, 0.0, 1.0);
    u_xlat16_44 = u_xlat16_44 + -0.0399999991;
    u_xlat16_41 = u_xlat16_41 * u_xlat16_44 + 0.0399999991;
    u_xlat16_44 = (-u_xlat16_10.x) * 0.699999988 + 1.70000005;
    u_xlat16_44 = u_xlat16_44 * u_xlat16_10.x;
    u_xlat16_10.x = u_xlat16_10.x * u_xlat16_10.x;
    u_xlat16_44 = u_xlat16_44 * 6.0;
    u_xlat16_5 = textureLod(unity_SpecCube0, u_xlat16_5.xyz, u_xlat16_44);
    u_xlat16_23.x = u_xlat16_5.w + -1.0;
    u_xlat16_10.y = unity_SpecCube0_HDR.w * u_xlat16_23.x + 1.0;
    u_xlat16_10.xy = max(u_xlat16_10.xy, vec2(0.0078125, 0.0));
    u_xlat16_23.x = log2(u_xlat16_10.y);
    u_xlat16_23.x = u_xlat16_23.x * unity_SpecCube0_HDR.y;
    u_xlat16_23.x = exp2(u_xlat16_23.x);
    u_xlat16_23.x = u_xlat16_23.x * unity_SpecCube0_HDR.x;
    u_xlat16_23.xyz = u_xlat16_5.xyz * u_xlat16_23.xxx;
    u_xlat16_11.xy = u_xlat16_10.xx * u_xlat16_10.xx + vec2(-1.0, 1.0);
    u_xlat16_24.x = float(1.0) / u_xlat16_11.y;
    u_xlat1.x = u_xlat16_41 * u_xlat16_24.x;
    u_xlat16_23.xyz = u_xlat1.xxx * u_xlat16_23.xyz;
    u_xlat16_24.xyz = u_xlat16_3.xyz * u_xlat16_9.yyy;
    u_xlat16_24.xyz = u_xlat16_0.xyz * u_xlat16_9.xxx + u_xlat16_24.xyz;
    u_xlat16_24.xyz = u_xlat16_8.xyz * u_xlat16_9.zzz + u_xlat16_24.xyz;
    u_xlat16_24.xyz = u_xlat16_7.xyz * u_xlat16_9.www + u_xlat16_24.xyz;
    u_xlat16_12.xyz = u_xlat16_0.xyz + (-u_xlat16_24.xyz);
    u_xlat16_41 = (-u_xlat16_9.x) + 1.0;
    u_xlat16_41 = (-u_xlat16_9.y) + u_xlat16_41;
    u_xlat16_41 = (-u_xlat16_9.z) + u_xlat16_41;
    u_xlat16_41 = (-u_xlat16_9.w) + u_xlat16_41;
    u_xlat16_41 = clamp(u_xlat16_41, 0.0, 1.0);
    SV_TARGET0.w = dot(u_xlat16_9, vec4(1.0, 1.0, 1.0, 1.0));
    u_xlat16_24.xyz = vec3(u_xlat16_41) * u_xlat16_12.xyz + u_xlat16_24.xyz;
    u_xlat16_24.xyz = u_xlat16_24.xyz * vec3(0.959999979, 0.959999979, 0.959999979);
    u_xlat16_23.xyz = vs_TEXCOORD8.xyz * u_xlat16_24.xyz + u_xlat16_23.xyz;
    u_xlat1.x = dot(u_xlat4.xyz, u_xlat4.xyz);
    u_xlat1.x = max(u_xlat1.x, 1.17549435e-38);
    u_xlat1.x = inversesqrt(u_xlat1.x);
    u_xlat3.xyz = u_xlat1.xxx * u_xlat4.xyz;
    u_xlat1.x = dot(u_xlat14.xyz, u_xlat3.xyz);
    u_xlat1.x = clamp(u_xlat1.x, 0.0, 1.0);
    u_xlat14.x = dot(_MainLightPosition.xyz, u_xlat3.xyz);
    u_xlat14.x = clamp(u_xlat14.x, 0.0, 1.0);
    u_xlat16_41 = u_xlat14.x * u_xlat14.x;
    u_xlat14.x = max(u_xlat16_41, 0.100000001);
    u_xlat1.x = u_xlat1.x * u_xlat1.x;
    u_xlat1.x = u_xlat1.x * u_xlat16_11.x + 1.00001001;
    u_xlat1.x = u_xlat1.x * u_xlat1.x;
    u_xlat1.x = u_xlat14.x * u_xlat1.x;
    u_xlat16_41 = u_xlat16_10.x * 4.0 + 2.0;
    u_xlat16_10.x = u_xlat16_10.x * u_xlat16_10.x;
    u_xlat1.x = u_xlat1.x * u_xlat16_41;
    u_xlat1.x = u_xlat16_10.x / u_xlat1.x;
    u_xlat16_41 = u_xlat1.x + -6.10351563e-05;
    u_xlat16_41 = max(u_xlat16_41, 0.0);
    u_xlat16_41 = min(u_xlat16_41, 100.0);
    u_xlat16_11.xyz = vec3(u_xlat16_41) * vec3(0.0399999991, 0.0399999991, 0.0399999991) + u_xlat16_24.xyz;
    SV_TARGET0.xyz = u_xlat16_11.xyz * u_xlat16_2.xyz + u_xlat16_23.xyz;
    return;
}

#endif
