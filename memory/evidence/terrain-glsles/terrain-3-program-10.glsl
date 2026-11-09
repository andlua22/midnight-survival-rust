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
uniform 	vec4 hlslcc_mtx4x4_MainLightWorldToShadow[20];
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
in highp vec2 in_TEXCOORD1;
out highp vec2 vs_TEXCOORD0;
out mediump float vs_TEXCOORD5;
out highp vec3 vs_TEXCOORD1;
out highp vec3 vs_TEXCOORD3;
out highp vec3 vs_TEXCOORD4;
out highp vec4 vs_TEXCOORD6;
out highp vec3 vs_TEXCOORD7;
out highp vec2 vs_TEXCOORD8;
out highp vec4 vs_TEXCOORD11;
vec4 u_xlat0;
vec4 u_xlat1;
vec3 u_xlat2;
float u_xlat9;
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
    u_xlat9 = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlat9 = max(u_xlat9, 1.17549435e-38);
    u_xlat9 = inversesqrt(u_xlat9);
    u_xlat1.xyz = vec3(u_xlat9) * u_xlat1.xyz;
    u_xlat9 = dot(u_xlat1.xyz, u_xlat1.xyz);
    u_xlat9 = inversesqrt(u_xlat9);
    vs_TEXCOORD3.xyz = vec3(u_xlat9) * u_xlat1.xyz;
    vs_TEXCOORD4.xyz = (-u_xlat0.xyz) + _WorldSpaceCameraPos.xyz;
    u_xlat1 = u_xlat0.yyyy * hlslcc_mtx4x4unity_MatrixVP[1];
    u_xlat1 = hlslcc_mtx4x4unity_MatrixVP[0] * u_xlat0.xxxx + u_xlat1;
    u_xlat1 = hlslcc_mtx4x4unity_MatrixVP[2] * u_xlat0.zzzz + u_xlat1;
    u_xlat1 = u_xlat1 + hlslcc_mtx4x4unity_MatrixVP[3];
    u_xlat9 = u_xlat1.z + _ProjectionParams.y;
    u_xlat2.x = _ProjectionParams.y + _ProjectionParams.z;
    u_xlat9 = u_xlat9 / u_xlat2.x;
    u_xlat9 = u_xlat9 * _ProjectionParams.z;
    u_xlat9 = max(u_xlat9, 0.0);
    u_xlat9 = u_xlat9 * unity_FogParams.z + unity_FogParams.w;
    u_xlat9 = clamp(u_xlat9, 0.0, 1.0);
    vs_TEXCOORD5 = u_xlat9;
    u_xlat2.xyz = u_xlat0.yyy * hlslcc_mtx4x4_MainLightWorldToShadow[1].xyz;
    u_xlat0.xyw = hlslcc_mtx4x4_MainLightWorldToShadow[0].xyz * u_xlat0.xxx + u_xlat2.xyz;
    u_xlat0.xyz = hlslcc_mtx4x4_MainLightWorldToShadow[2].xyz * u_xlat0.zzz + u_xlat0.xyw;
    vs_TEXCOORD6.xyz = u_xlat0.xyz + hlslcc_mtx4x4_MainLightWorldToShadow[3].xyz;
    vs_TEXCOORD6.w = 0.0;
    vs_TEXCOORD7.xyz = vec3(0.0, 0.0, 0.0);
    vs_TEXCOORD8.xy = in_TEXCOORD1.xy * unity_LightmapST.xy + unity_LightmapST.zw;
    gl_Position = u_xlat1;
    u_xlat0.x = u_xlat1.y * _ProjectionParams.x;
    u_xlat0.w = u_xlat0.x * 0.5;
    u_xlat0.xz = u_xlat1.xw * vec2(0.5, 0.5);
    vs_TEXCOORD11.zw = u_xlat1.zw;
    vs_TEXCOORD11.xy = u_xlat0.zz + u_xlat0.xw;
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
uniform 	vec4 _ProjectionParams;
uniform 	vec2 _GlobalMipBias;
uniform 	vec4 unity_FogParams;
uniform 	mediump vec4 unity_FogColor;
uniform 	vec4 hlslcc_mtx4x4unity_MatrixV[4];
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
UNITY_LOCATION(0) uniform mediump samplerCube unity_SpecCube0;
UNITY_LOCATION(1) uniform mediump sampler2D unity_Lightmap;
UNITY_LOCATION(2) uniform mediump sampler2D _Control;
UNITY_LOCATION(3) uniform mediump sampler2D _Splat0;
UNITY_LOCATION(4) uniform mediump sampler2D _Splat1;
UNITY_LOCATION(5) uniform mediump sampler2D _Splat2;
UNITY_LOCATION(6) uniform mediump sampler2D _MainLightShadowmapTexture;
UNITY_LOCATION(7) uniform mediump sampler2DShadow hlslcc_zcmp_MainLightShadowmapTexture;
in highp vec2 vs_TEXCOORD0;
in highp vec3 vs_TEXCOORD1;
in highp vec3 vs_TEXCOORD3;
in highp vec3 vs_TEXCOORD4;
in highp vec4 vs_TEXCOORD6;
in highp vec2 vs_TEXCOORD8;
layout(location = 0) out mediump vec4 SV_TARGET0;
vec3 u_xlat0;
mediump vec4 u_xlat16_0;
bool u_xlatb0;
mediump vec3 u_xlat16_1;
vec3 u_xlat2;
vec3 u_xlat3;
mediump vec4 u_xlat16_4;
mediump vec3 u_xlat16_5;
vec4 u_xlat6;
mediump vec4 u_xlat16_6;
mediump vec4 u_xlat16_7;
vec2 u_xlat8;
mediump vec4 u_xlat16_8;
mediump vec3 u_xlat16_9;
mediump vec3 u_xlat16_10;
vec3 u_xlat11;
bool u_xlatb11;
mediump vec3 u_xlat16_16;
mediump vec3 u_xlat16_20;
float u_xlat33;
mediump float u_xlat16_34;
mediump float u_xlat16_37;
mediump float u_xlat16_38;
void main()
{
    u_xlat16_0.xyz = texture(unity_Lightmap, vs_TEXCOORD8.xy, _GlobalMipBias.x).xyz;
    u_xlat16_1.xyz = u_xlat16_0.xyz * vec3(4.59000015, 4.59000015, 4.59000015);
    u_xlat0.x = dot(vs_TEXCOORD4.xyz, vs_TEXCOORD4.xyz);
    u_xlat0.x = max(u_xlat0.x, 1.17549435e-38);
    u_xlat0.x = inversesqrt(u_xlat0.x);
    u_xlat11.xyz = u_xlat0.xxx * vs_TEXCOORD4.xyz;
    u_xlat2.xyz = vs_TEXCOORD4.xyz * u_xlat0.xxx + _MainLightPosition.xyz;
    u_xlat0.x = dot(vs_TEXCOORD3.xyz, vs_TEXCOORD3.xyz);
    u_xlat0.x = inversesqrt(u_xlat0.x);
    u_xlat3.xyz = u_xlat0.xxx * vs_TEXCOORD3.xyz;
    u_xlat16_34 = dot((-u_xlat11.xyz), u_xlat3.xyz);
    u_xlat16_34 = u_xlat16_34 + u_xlat16_34;
    u_xlat16_4.xyz = u_xlat3.xyz * (-vec3(u_xlat16_34)) + (-u_xlat11.xyz);
    u_xlat16_34 = dot(u_xlat3.xyz, u_xlat11.xyz);
    u_xlat16_34 = clamp(u_xlat16_34, 0.0, 1.0);
    u_xlat16_34 = (-u_xlat16_34) + 1.0;
    u_xlat16_34 = u_xlat16_34 * u_xlat16_34;
    u_xlat16_34 = u_xlat16_34 * u_xlat16_34;
    u_xlat0.xy = vs_TEXCOORD0.xy * vec2(vec2(_Splat2_S, _Splat2_S));
    u_xlat16_0 = texture(_Splat2, u_xlat0.xy, _GlobalMipBias.x);
    u_xlat16_5.z = u_xlat16_0.w * _Splat2_Smoothness;
    u_xlat6 = vs_TEXCOORD0.xyxy * vec4(_Splat0_S, _Splat0_S, _Splat1_S, _Splat1_S);
    u_xlat16_7 = texture(_Splat1, u_xlat6.zw, _GlobalMipBias.x);
    u_xlat16_6 = texture(_Splat0, u_xlat6.xy, _GlobalMipBias.x);
    u_xlat8.y = u_xlat16_7.w;
    u_xlat8.x = u_xlat16_6.w;
    u_xlat16_5.xy = u_xlat8.xy * vec2(_Splat0_Smoothness, _Splat1_Smoothness);
    u_xlat16_8 = texture(_Control, vs_TEXCOORD0.xy, _GlobalMipBias.x);
    u_xlat16_37 = dot(u_xlat16_8.xyz, u_xlat16_5.xyz);
    u_xlat16_5.x = (-u_xlat16_37) + 1.0;
    u_xlat16_37 = u_xlat16_37 + 0.0400000215;
    u_xlat16_37 = clamp(u_xlat16_37, 0.0, 1.0);
    u_xlat16_37 = u_xlat16_37 + -0.0399999991;
    u_xlat16_34 = u_xlat16_34 * u_xlat16_37 + 0.0399999991;
    u_xlat16_37 = (-u_xlat16_5.x) * 0.699999988 + 1.70000005;
    u_xlat16_37 = u_xlat16_37 * u_xlat16_5.x;
    u_xlat16_5.x = u_xlat16_5.x * u_xlat16_5.x;
    u_xlat16_37 = u_xlat16_37 * 6.0;
    u_xlat16_4 = textureLod(unity_SpecCube0, u_xlat16_4.xyz, u_xlat16_37);
    u_xlat16_16.x = u_xlat16_4.w + -1.0;
    u_xlat16_5.y = unity_SpecCube0_HDR.w * u_xlat16_16.x + 1.0;
    u_xlat16_5.xy = max(u_xlat16_5.xy, vec2(0.0078125, 0.0));
    u_xlat16_16.x = log2(u_xlat16_5.y);
    u_xlat16_16.x = u_xlat16_16.x * unity_SpecCube0_HDR.y;
    u_xlat16_16.x = exp2(u_xlat16_16.x);
    u_xlat16_16.x = u_xlat16_16.x * unity_SpecCube0_HDR.x;
    u_xlat16_16.xyz = u_xlat16_4.xyz * u_xlat16_16.xxx;
    u_xlat16_9.xy = u_xlat16_5.xx * u_xlat16_5.xx + vec2(-1.0, 1.0);
    u_xlat16_20.x = float(1.0) / u_xlat16_9.y;
    u_xlat33 = u_xlat16_34 * u_xlat16_20.x;
    u_xlat16_16.xyz = vec3(u_xlat33) * u_xlat16_16.xyz;
    u_xlat16_20.xyz = u_xlat16_7.xyz * u_xlat16_8.yyy;
    u_xlat16_20.xyz = u_xlat16_6.xyz * u_xlat16_8.xxx + u_xlat16_20.xyz;
    u_xlat16_20.xyz = u_xlat16_0.xyz * u_xlat16_8.zzz + u_xlat16_20.xyz;
    u_xlat16_10.xyz = u_xlat16_6.xyz + (-u_xlat16_20.xyz);
    u_xlat16_34 = (-u_xlat16_8.x) + 1.0;
    u_xlat16_34 = (-u_xlat16_8.y) + u_xlat16_34;
    u_xlat16_34 = (-u_xlat16_8.z) + u_xlat16_34;
    u_xlat16_34 = (-u_xlat16_8.w) + u_xlat16_34;
    u_xlat16_34 = clamp(u_xlat16_34, 0.0, 1.0);
    SV_TARGET0.w = dot(u_xlat16_8, vec4(1.0, 1.0, 1.0, 1.0));
    u_xlat16_20.xyz = vec3(u_xlat16_34) * u_xlat16_10.xyz + u_xlat16_20.xyz;
    u_xlat16_20.xyz = u_xlat16_20.xyz * vec3(0.959999979, 0.959999979, 0.959999979);
    u_xlat16_1.xyz = u_xlat16_1.xyz * u_xlat16_20.xyz + u_xlat16_16.xyz;
    u_xlat0.x = dot(u_xlat2.xyz, u_xlat2.xyz);
    u_xlat0.x = max(u_xlat0.x, 1.17549435e-38);
    u_xlat0.x = inversesqrt(u_xlat0.x);
    u_xlat0.xyz = u_xlat0.xxx * u_xlat2.xyz;
    u_xlat33 = dot(_MainLightPosition.xyz, u_xlat0.xyz);
    u_xlat33 = clamp(u_xlat33, 0.0, 1.0);
    u_xlat0.x = dot(u_xlat3.xyz, u_xlat0.xyz);
    u_xlat0.x = clamp(u_xlat0.x, 0.0, 1.0);
    u_xlat16_34 = dot(u_xlat3.xyz, _MainLightPosition.xyz);
    u_xlat16_34 = clamp(u_xlat16_34, 0.0, 1.0);
    u_xlat0.x = u_xlat0.x * u_xlat0.x;
    u_xlat0.x = u_xlat0.x * u_xlat16_9.x + 1.00001001;
    u_xlat0.x = u_xlat0.x * u_xlat0.x;
    u_xlat16_16.x = u_xlat33 * u_xlat33;
    u_xlat11.x = max(u_xlat16_16.x, 0.100000001);
    u_xlat0.x = u_xlat11.x * u_xlat0.x;
    u_xlat16_16.x = u_xlat16_5.x * 4.0 + 2.0;
    u_xlat16_5.x = u_xlat16_5.x * u_xlat16_5.x;
    u_xlat0.x = u_xlat0.x * u_xlat16_16.x;
    u_xlat0.x = u_xlat16_5.x / u_xlat0.x;
    u_xlat16_5.x = u_xlat0.x + -6.10351563e-05;
    u_xlat16_5.x = max(u_xlat16_5.x, 0.0);
    u_xlat16_5.x = min(u_xlat16_5.x, 100.0);
    u_xlat16_5.xyz = u_xlat16_5.xxx * vec3(0.0399999991, 0.0399999991, 0.0399999991) + u_xlat16_20.xyz;
    vec3 txVec0 = vec3(vs_TEXCOORD6.xy,vs_TEXCOORD6.z);
    u_xlat16_0.x = textureLod(hlslcc_zcmp_MainLightShadowmapTexture, txVec0, 0.0);
    u_xlat16_38 = (-_MainLightShadowParams.x) + 1.0;
    u_xlat16_38 = u_xlat16_0.x * _MainLightShadowParams.x + u_xlat16_38;
    u_xlatb0 = 0.0>=vs_TEXCOORD6.z;
    u_xlatb11 = vs_TEXCOORD6.z>=1.0;
    u_xlatb0 = u_xlatb11 || u_xlatb0;
    u_xlat16_38 = (u_xlatb0) ? 1.0 : u_xlat16_38;
    u_xlat16_9.x = (-u_xlat16_38) + 1.0;
    u_xlat0.xyz = vs_TEXCOORD1.xyz + (-_WorldSpaceCameraPos.xyz);
    u_xlat0.x = dot(u_xlat0.xyz, u_xlat0.xyz);
    u_xlat0.x = u_xlat0.x * _MainLightShadowParams.z + _MainLightShadowParams.w;
    u_xlat0.x = clamp(u_xlat0.x, 0.0, 1.0);
    u_xlat16_38 = u_xlat0.x * u_xlat16_9.x + u_xlat16_38;
    u_xlat0.x = u_xlat16_38 * unity_LightData.z;
    u_xlat16_34 = u_xlat16_34 * u_xlat0.x;
    u_xlat16_9.xyz = vec3(u_xlat16_34) * _MainLightColor.xyz;
    u_xlat16_1.xyz = u_xlat16_5.xyz * u_xlat16_9.xyz + u_xlat16_1.xyz;
    u_xlat16_1.xyz = u_xlat16_1.xyz + (-unity_FogColor.xyz);
    u_xlat0.x = vs_TEXCOORD1.y * hlslcc_mtx4x4unity_MatrixV[1].z;
    u_xlat0.x = hlslcc_mtx4x4unity_MatrixV[0].z * vs_TEXCOORD1.x + u_xlat0.x;
    u_xlat0.x = hlslcc_mtx4x4unity_MatrixV[2].z * vs_TEXCOORD1.z + u_xlat0.x;
    u_xlat0.x = u_xlat0.x + hlslcc_mtx4x4unity_MatrixV[3].z;
    u_xlat0.x = (-u_xlat0.x) + (-_ProjectionParams.y);
    u_xlat0.x = max(u_xlat0.x, 0.0);
    u_xlat0.x = u_xlat0.x * unity_FogParams.z + unity_FogParams.w;
    u_xlat0.x = clamp(u_xlat0.x, 0.0, 1.0);
    SV_TARGET0.xyz = u_xlat0.xxx * u_xlat16_1.xyz + unity_FogColor.xyz;
    return;
}

#endif
