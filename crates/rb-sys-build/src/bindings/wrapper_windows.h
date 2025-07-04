// Windows-specific wrapper to prevent AVX512 intrinsics issues
// This file is included BEFORE the main wrapper.h content on Windows

#ifdef _WIN32
  // Step 0: Define the problematic types FIRST before any headers can define them differently
  typedef struct { char dummy[16]; } __m128h;
  typedef struct { char dummy[32]; } __m256h;
  typedef struct { char dummy[64]; } __m512h;
  
  // Also define the vector types that might be used
  typedef struct { char dummy[16]; } __v8hf;
  typedef struct { char dummy[32]; } __v16hf;
  typedef struct { char dummy[64]; } __v32hf;
  
  // Define _Float16 as a dummy type
  typedef struct { char dummy[2]; } _Float16;

  // Step 1: Define all intrinsics header guards to prevent their inclusion
  // This must be done before ANY includes
  #define _IMMINTRIN_H
  #define _AMXAVX512INTRIN_H
  #define _AVX10_2CONVERTINTRIN_H
  #define _AVX512FP16INTRIN_H
  #define _AVX512VLFP16INTRIN_H
  #define _AVX512FINTRIN_H
  #define _AVX512PFINTRIN_H
  #define _AVX512VLINTRIN_H
  #define _AVX512BWINTRIN_H
  #define _AVX512DQINTRIN_H
  #define _AVX512CDINTRIN_H
  #define _AVX512ERINTRIN_H
  #define _AVX512IFMAINTRIN_H
  #define _AVX512IFMAVLINTRIN_H
  #define _AVX512VBMIINTRIN_H
  #define _AVX512VBMIVLINTRIN_H
  #define _AVX512VBMI2INTRIN_H
  #define _AVX512VBMI2VLINTRIN_H
  #define _AVX512VNNIINTRIN_H
  #define _AVX512VNNIVLINTRIN_H
  #define _AVX512VPOPCNTDQINTRIN_H
  #define _AVX512VPOPCNTDQVLINTRIN_H
  #define _AVX512BITALGINTRIN_H
  #define _AVX512BITALG_H
  #define _AVX512BF16INTRIN_H
  #define _AVX512BF16VLINTRIN_H
  #define _AVX512VP2INTERSECTINTRIN_H
  #define _AVX512VP2INTERSECTVLINTRIN_H
  #define _AVX10_1_256INTRIN_H
  #define _AVX10_1_512INTRIN_H
  #define _AVX10_1INTRIN_H
  #define _AVX10_2_256INTRIN_H
  #define _AVX10_2_512INTRIN_H
  #define _AVX10_2INTRIN_H
  #define _AVX10_2SATCVTINTRIN_H
  #define _AVX10_2COPYINTRIN_H
  #define _AVX10_2MEDIAINTRIN_H
  #define _AVX10_2MINMAXINTRIN_H
  
  // Also prevent emmintrin.h and avxintrin.h
  #define _EMMINTRIN_H
  #define _AVXINTRIN_H
  #define _AVX2INTRIN_H
  #define _XMMINTRIN_H
  #define _PMMINTRIN_H
  #define _TMMINTRIN_H
  #define _SMMINTRIN_H
  #define _NMMINTRIN_H
  
  // Step 2: Undefine all CPU feature macros that would trigger intrinsics
  #ifdef __AVX512F__
    #undef __AVX512F__
  #endif
  #ifdef __AVX512FP16__
    #undef __AVX512FP16__
  #endif
  #ifdef __AMX_AVX512__
    #undef __AMX_AVX512__
  #endif
  #ifdef __AVX10_1__
    #undef __AVX10_1__
  #endif
  #ifdef __AVX10_1_512__
    #undef __AVX10_1_512__
  #endif
  #ifdef __AVX10_2__
    #undef __AVX10_2__
  #endif
  #ifdef __AVX10_2_512__
    #undef __AVX10_2_512__
  #endif
  #ifdef __AVX__
    #undef __AVX__
  #endif
  #ifdef __AVX2__
    #undef __AVX2__
  #endif
#endif // _WIN32