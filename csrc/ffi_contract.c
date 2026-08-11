#include <stddef.h>
#include "crapto1.h"
#include "hardnested_entry.h"

size_t crapto1_cnonce_size(void) {
  return sizeof(CNonce);
}

size_t crapto1_cnonce_align(void) {
  return _Alignof(CNonce);
}

int32_t crapto1_attack_value(int which) {
  switch (which) {
    case 0:
      return ATTACK_MFKEY32;
    case 1:
      return ATTACK_STATIC_NESTED;
    case 2:
      return ATTACK_STATIC_ENCRYPTED;
    default:
      return -1;
  }
}

size_t crapto1_cnonce_offset(int field) {
  switch (field) {
    case 0:
      return offsetof(CNonce, attack);
    case 1:
      return offsetof(CNonce, key_idx);
    case 2:
      return offsetof(CNonce, uid);
    case 3:
      return offsetof(CNonce, nt0);
    case 4:
      return offsetof(CNonce, nt1);
    case 5:
      return offsetof(CNonce, uid_xor_nt0);
    case 6:
      return offsetof(CNonce, uid_xor_nt1);
    case 7:
      return offsetof(CNonce, p64);
    case 8:
      return offsetof(CNonce, p64b);
    case 9:
      return offsetof(CNonce, nr0_enc);
    case 10:
      return offsetof(CNonce, ar0_enc);
    case 11:
      return offsetof(CNonce, nr1_enc);
    case 12:
      return offsetof(CNonce, ar1_enc);
    case 13:
      return offsetof(CNonce, ks1_1_enc);
    case 14:
      return offsetof(CNonce, ks1_2_enc);
    case 15:
      return offsetof(CNonce, par_1);
    case 16:
      return offsetof(CNonce, par_2);
    default:
      return (size_t)-1;
  }
}

size_t hn_nonce_size(void) {
  return sizeof(HnNonce);
}

size_t hn_nonce_align(void) {
  return _Alignof(HnNonce);
}

size_t hn_nonce_offset(int field) {
  switch (field) {
    case 0:
      return offsetof(HnNonce, nt_enc);
    case 1:
      return offsetof(HnNonce, par);
    default:
      return (size_t)-1;
  }
}

size_t hn_callbacks_size(void) {
  return sizeof(HnCallbacks);
}

size_t hn_callbacks_align(void) {
  return _Alignof(HnCallbacks);
}

size_t hn_callbacks_offset(int field) {
  switch (field) {
    case 0:
      return offsetof(HnCallbacks, line);
    case 1:
      return offsetof(HnCallbacks, user);
    default:
      return (size_t)-1;
  }
}
