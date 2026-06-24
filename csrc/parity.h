#ifndef __PARITY_H
#define __PARITY_H

#include <stdint.h>

static inline uint8_t evenparity8(uint8_t x) {
#if defined(_MSC_VER)
  x ^= x >> 4;
  x ^= x >> 2;
  x ^= x >> 1;
  return (uint8_t)(x & 1);
#else
  return (uint8_t)__builtin_parity((unsigned int)x);
#endif
}

static inline uint8_t evenparity32(uint32_t x) {
#if defined(_MSC_VER)
  x ^= x >> 16;
  x ^= x >> 8;
  x ^= x >> 4;
  x ^= x >> 2;
  x ^= x >> 1;
  return (uint8_t)(x & 1);
#else
  return (uint8_t)__builtin_parity(x);
#endif
}

#endif /* __PARITY_H */