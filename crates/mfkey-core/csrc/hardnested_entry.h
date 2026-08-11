#ifndef HARDNESTED_ENTRY_H
#define HARDNESTED_ENTRY_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
  uint32_t nt_enc;
  uint8_t par;
} HnNonce;

typedef struct {
  void (*line)(const char *text, void *user);
  void *user;
} HnCallbacks;

int hardnested_recover(uint32_t uid, uint8_t key_type, const HnNonce *nonces,
                       uint32_t count, const HnCallbacks *cb, uint64_t *out_key);

#ifdef __cplusplus
}
#endif

#endif
