#ifndef MFKEY_MSVC_COMPAT_H
#define MFKEY_MSVC_COMPAT_H

#ifdef _MSC_VER

#include <intrin.h>

#ifndef restrict
#define restrict __restrict
#endif

#define __attribute__(x)

#ifndef __builtin_assume_aligned
#define __builtin_assume_aligned(p, a) (p)
#endif

static __inline int mfkey_builtin_clz(unsigned int x) {
  unsigned long idx;
  if (_BitScanReverse(&idx, x)) {
    return (int)(31 - idx);
  }
  return 32;
}

#ifndef __builtin_clz
#define __builtin_clz(x) mfkey_builtin_clz((unsigned int)(x))
#endif

#endif

#endif
