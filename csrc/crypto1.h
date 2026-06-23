#ifndef CRYPTO1_H
#define CRYPTO1_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MF_CLASSIC_KEY_SIZE 6

typedef enum {
  ATTACK_MFKEY32 = 0,
  ATTACK_STATIC_NESTED = 1,
  ATTACK_STATIC_ENCRYPTED = 2
} CAttackType;

typedef struct {
  int32_t attack;
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
  void (*candidate_key)(const uint8_t *key6, void *user);
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