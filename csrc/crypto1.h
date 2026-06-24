#ifndef CRYPTO1_H
#define CRYPTO1_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LF_POLY_ODD (0x29CE5C)
#define LF_POLY_EVEN (0x870804)
#define CONST_M1_1 (LF_POLY_EVEN << 1 | 1)
#define CONST_M2_1 (LF_POLY_ODD << 1)
#define CONST_M1_2 (LF_POLY_ODD)
#define CONST_M2_2 (LF_POLY_EVEN << 1 | 1)
#define BIT(x, n) ((x) >> (n) & 1)
#define BEBIT(x, n) BIT(x, (n) ^ 24)
#define SWAPENDIAN(x)                                                          \
  (x = (x >> 8 & 0xff00ff) | (x & 0xff00ff) << 8, x = x >> 16 | x << 16)

#ifdef __OPTIMIZE_SIZE__
int filter(uint32_t const x);
#else
static inline int filter(uint32_t const x) {
  uint32_t f;

  f = 0xf22c0 >> (x & 0xf) & 16;
  f |= 0x6c9c0 >> (x >> 4 & 0xf) & 8;
  f |= 0x3c8b0 >> (x >> 8 & 0xf) & 4;
  f |= 0x1e458 >> (x >> 12 & 0xf) & 2;
  f |= 0x0d938 >> (x >> 16 & 0xf) & 1;
  return BIT(0xEC57E80A, f);
}
#endif

#define MF_CLASSIC_KEY_SIZE 6

typedef enum {
  ATTACK_MFKEY32 = 0,
  ATTACK_STATIC_NESTED = 1,
  ATTACK_STATIC_ENCRYPTED = 2
} CAttackType;

typedef struct {
  int32_t attack;
  uint8_t key_idx;
  uint32_t uid;
  uint32_t nt0;
  uint32_t nt1;
  uint32_t uid_xor_nt0;
  uint32_t uid_xor_nt1;

  uint32_t p64;
  uint32_t p64b;
  uint32_t nr0_enc;
  uint32_t ar0_enc;
  uint32_t nr1_enc;
  uint32_t ar1_enc;

  uint32_t ks1_1_enc;
  uint32_t ks1_2_enc;
  uint8_t par_1;
  uint8_t par_2;
} CNonce;

typedef struct {
  void (*found_key)(const uint8_t *key6, void *user);
  void (*candidate_key)(const uint8_t *key6, uint8_t key_idx, void *user);
  void (*progress)(uint32_t msb_round, uint32_t total_rounds,
                   float stage_progress, uint32_t uid, void *user);
  int (*should_stop)(void *user);
  void *user;
} CCallbacks;

bool crypto1_recover(const CNonce *n, uint32_t ks2, uint32_t in,
                     const CCallbacks *cb);

uint32_t crypto1_prng_successor(uint32_t x, uint32_t n);

#ifdef __cplusplus
}
#endif

#endif // CRYPTO1_H