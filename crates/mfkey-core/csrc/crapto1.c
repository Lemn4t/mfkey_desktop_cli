#include "crapto1.h"
#include <stdlib.h>
#include <string.h>

#include "parity.h"

#define MSB_LIMIT_BASE 16

#if defined(_MSC_VER)
#define MFKEY_THREAD_LOCAL __declspec(thread)
#else
#define MFKEY_THREAD_LOCAL __thread
#endif

struct Crypto1State {
  uint32_t odd, even;
};

struct Msb {
  int tail;
  uint32_t states[768];
};

#define MSB_ARRAY_LEN (MSB_LIMIT_BASE * 2)

static MFKEY_THREAD_LOCAL struct Msb tls_odd_msbs[MSB_ARRAY_LEN];
static MFKEY_THREAD_LOCAL struct Msb tls_even_msbs[MSB_ARRAY_LEN];
static MFKEY_THREAD_LOCAL unsigned int tls_temp_states_odd[1280];
static MFKEY_THREAD_LOCAL unsigned int tls_temp_states_even[1280];
static MFKEY_THREAD_LOCAL unsigned int tls_states_buffer[1024];

typedef struct {
  const CNonce *n;
  const CCallbacks *cb;
  int current_msb_round;
  int total_msb_rounds;
} RecoverCtx;

static uint8_t get_nth_byte(uint32_t value, int n) {
  if (n < 0 || n > 3) {
    return 0;
  }
  return (value >> (8 * (3 - n))) & 0xFF;
}

static uint8_t crypto1_bit(struct Crypto1State *s, uint8_t in, int is_encrypted) {
  uint32_t feedin, t;
  uint8_t ret = filter(s->odd);

  feedin = ret & !!is_encrypted;
  feedin ^= !!in;
  feedin ^= LF_POLY_ODD & s->odd;
  feedin ^= LF_POLY_EVEN & s->even;
  s->even = s->even << 1 | evenparity32(feedin);

  t = s->odd;
  s->odd = s->even;
  s->even = t;

  return ret;
}

static inline uint32_t crypt_word_par(struct Crypto1State *s, uint32_t in, int is_encrypted,
                                      uint32_t nt_plain, uint8_t *parity_keystream_bits) {
  uint32_t ret = 0;
  *parity_keystream_bits = 0;

  for (int i = 0; i < 32; i++) {
    uint8_t bit = crypto1_bit(s, BEBIT(in, i), is_encrypted);
    ret |= bit << (24 ^ i);
    if ((i + 1) % 8 == 0) {
      *parity_keystream_bits |= (filter(s->odd) ^ evenparity8(get_nth_byte(nt_plain, i / 8)))
                                << (3 - (i / 8));
    }
  }
  return ret;
}

static inline void update_contribution(unsigned int data[], int item, int mask1, int mask2) {
  int p = data[item] >> 25;
  p = p << 1 | evenparity32(data[item] & mask1);
  p = p << 1 | evenparity32(data[item] & mask2);
  data[item] = p << 24 | (data[item] & 0xffffff);
}

static inline uint32_t crypt_word(struct Crypto1State *s) {
  uint32_t res_ret = 0;
  uint32_t feedin, t;
  for (int i = 0; i <= 31; i++) {
    res_ret |= (filter(s->odd) << (24 ^ i));
    feedin = LF_POLY_EVEN & s->even;
    feedin ^= LF_POLY_ODD & s->odd;
    s->even = s->even << 1 | (evenparity32(feedin));
    t = s->odd, s->odd = s->even, s->even = t;
  }
  return res_ret;
}

static inline void crypt_word_noret(struct Crypto1State *s, uint32_t in, int x) {
  uint8_t ret;
  uint32_t feedin, t, next_in;
  for (int i = 0; i <= 31; i++) {
    next_in = BEBIT(in, i);
    ret = filter(s->odd);
    feedin = ret & (!!x);
    feedin ^= LF_POLY_EVEN & s->even;
    feedin ^= LF_POLY_ODD & s->odd;
    feedin ^= !!next_in;
    s->even = s->even << 1 | (evenparity32(feedin));
    t = s->odd, s->odd = s->even, s->even = t;
  }
  (void)ret;
}

static inline uint32_t crypt_word_ret(struct Crypto1State *s, uint32_t in, int x) {
  uint32_t ret = 0;
  uint32_t feedin, t, next_in;
  uint8_t next_ret;
  for (int i = 0; i <= 31; i++) {
    next_in = BEBIT(in, i);
    next_ret = filter(s->odd);
    feedin = next_ret & (!!x);
    feedin ^= LF_POLY_EVEN & s->even;
    feedin ^= LF_POLY_ODD & s->odd;
    feedin ^= !!next_in;
    s->even = s->even << 1 | (evenparity32(feedin));
    t = s->odd, s->odd = s->even, s->even = t;
    ret |= next_ret << (24 ^ i);
  }
  return ret;
}

static inline void rollback_word_noret(struct Crypto1State *s, uint32_t in, int x) {
  uint8_t ret;
  uint32_t feedin, t, next_in;
  for (int i = 31; i >= 0; i--) {
    next_in = BEBIT(in, i);
    s->odd &= 0xffffff;
    t = s->odd, s->odd = s->even, s->even = t;
    ret = filter(s->odd);
    feedin = ret & (!!x);
    feedin ^= s->even & 1;
    feedin ^= LF_POLY_EVEN & (s->even >>= 1);
    feedin ^= LF_POLY_ODD & s->odd;
    feedin ^= !!next_in;
    s->even |= (evenparity32(feedin)) << 23;
  }
  (void)ret;
}

static uint8_t napi_lfsr_rollback_bit(struct Crypto1State *s, uint32_t in, int fb) {
  int out;
  uint8_t ret;
  uint32_t t;
  s->odd &= 0xffffff;
  t = s->odd, s->odd = s->even, s->even = t;

  out = s->even & 1;
  out ^= LF_POLY_EVEN & (s->even >>= 1);
  out ^= LF_POLY_ODD & s->odd;
  out ^= !!in;
  out ^= (ret = filter(s->odd)) & !!fb;

  s->even |= evenparity32(out) << 23;
  return ret;
}

static uint32_t napi_lfsr_rollback_word(struct Crypto1State *s, uint32_t in, int fb) {
  int i;
  uint32_t ret = 0;
  for (i = 31; i >= 0; --i)
    ret |= napi_lfsr_rollback_bit(s, BEBIT(in, i), fb) << (i ^ 24);
  return ret;
}

static void mfkey_state_to_key6(struct Crypto1State *state, uint8_t *lfsr_out6) {
  int i;
  uint64_t lfsr_value = 0;
  for (i = 23; i >= 0; --i) {
    lfsr_value = lfsr_value << 1 | BIT(state->odd, i ^ 3);
    lfsr_value = lfsr_value << 1 | BIT(state->even, i ^ 3);
  }
  for (i = 0; i < 6; ++i) {
    lfsr_out6[i] = (lfsr_value >> ((5 - i) * 8)) & 0xFF;
  }
}

static inline int check_state(struct Crypto1State *t, RecoverCtx *ctx) {
  if (!(t->odd | t->even))
    return 0;

  const CNonce *n = ctx->n;
  uint8_t key6[MF_CLASSIC_KEY_SIZE];

  if (n->attack == ATTACK_MFKEY32) {
    uint32_t rb = (napi_lfsr_rollback_word(t, 0, 0) ^ n->p64);
    if (rb != n->ar0_enc) {
      return 0;
    }
    rollback_word_noret(t, n->nr0_enc, 1);
    rollback_word_noret(t, n->uid_xor_nt0, 0);
    struct Crypto1State temp = {t->odd, t->even};
    crypt_word_noret(t, n->uid_xor_nt1, 0);
    crypt_word_noret(t, n->nr1_enc, 1);
    if (n->ar1_enc == (crypt_word(t) ^ n->p64b)) {
      mfkey_state_to_key6(&temp, key6);
      if (ctx->cb->found_key)
        ctx->cb->found_key(key6, ctx->cb->user);
      return 1;
    }
  } else if (n->attack == ATTACK_STATIC_NESTED) {
    struct Crypto1State temp = {t->odd, t->even};
    rollback_word_noret(t, n->uid_xor_nt1, 0);
    if (n->ks1_1_enc == crypt_word_ret(t, n->uid_xor_nt0, 0)) {
      rollback_word_noret(&temp, n->uid_xor_nt1, 0);
      mfkey_state_to_key6(&temp, key6);
      if (ctx->cb->found_key)
        ctx->cb->found_key(key6, ctx->cb->user);
      return 1;
    }
  } else if (n->attack == ATTACK_STATIC_ENCRYPTED) {
    if (n->ks1_1_enc == napi_lfsr_rollback_word(t, n->uid_xor_nt0, 0)) {
      uint8_t local_parity_keystream_bits;
      struct Crypto1State temp = {t->odd, t->even};
      if ((crypt_word_par(&temp, n->uid_xor_nt0, 0, n->nt0, &local_parity_keystream_bits) ==
           n->ks1_1_enc) &&
          (local_parity_keystream_bits == n->par_1)) {
        mfkey_state_to_key6(t, key6);
        if (ctx->cb->candidate_key)
          ctx->cb->candidate_key(key6, n->key_idx, ctx->cb->user);
      }
    }
  }
  return 0;
}

static inline int state_loop(unsigned int *states_buffer, int xks, int m1, int m2, unsigned int in,
                             uint8_t and_val) {
  int states_tail = 0;
  int round = 0, s = 0, xks_bit = 0, round_in = 0;

  for (round = 1; round <= 12; round++) {
    xks_bit = BIT(xks, round);
    if (round > 4) {
      round_in = ((in >> (2 * (round - 4))) & and_val) << 24;
    }

    for (s = 0; s <= states_tail; s++) {
      states_buffer[s] <<= 1;

      if ((filter(states_buffer[s]) ^ filter(states_buffer[s] | 1)) != 0) {
        states_buffer[s] |= filter(states_buffer[s]) ^ xks_bit;
        if (round > 4) {
          update_contribution(states_buffer, s, m1, m2);
          states_buffer[s] ^= round_in;
        }
      } else if (filter(states_buffer[s]) == xks_bit) {
        if (round > 4) {
          states_buffer[++states_tail] = states_buffer[s + 1];
          states_buffer[s + 1] = states_buffer[s] | 1;
          update_contribution(states_buffer, s, m1, m2);
          states_buffer[s++] ^= round_in;
          update_contribution(states_buffer, s, m1, m2);
          states_buffer[s] ^= round_in;
        } else {
          states_buffer[++states_tail] = states_buffer[++s];
          states_buffer[s] = states_buffer[s - 1] | 1;
        }
      } else {
        states_buffer[s--] = states_buffer[states_tail--];
      }
    }
  }

  return states_tail;
}

static int binsearch(unsigned int data[], int start, int stop) {
  int mid, val = data[stop] & 0xff000000;
  while (start != stop) {
    mid = (stop - start) >> 1;
    if ((data[start + mid] ^ 0x80000000) > (val ^ 0x80000000))
      stop = start + mid;
    else
      start += mid + 1;
  }
  return start;
}

static void quicksort(unsigned int array[], int low, int high) {
  if (low >= high)
    return;
  int middle = low + (high - low) / 2;
  unsigned int pivot = array[middle];
  int i = low, j = high;
  while (i <= j) {
    while (array[i] < pivot) {
      i++;
    }
    while (array[j] > pivot) {
      j--;
    }
    if (i <= j) {
      unsigned int temp = array[i];
      array[i] = array[j];
      array[j] = temp;
      i++;
      j--;
    }
  }
  if (low < j) {
    quicksort(array, low, j);
  }
  if (high > i) {
    quicksort(array, i, high);
  }
}

static int extend_table(unsigned int data[], int tbl, int end, int bit, int m1, int m2,
                        unsigned int in) {
  in <<= 24;
  for (data[tbl] <<= 1; tbl <= end; data[++tbl] <<= 1) {
    if ((filter(data[tbl]) ^ filter(data[tbl] | 1)) != 0) {
      data[tbl] |= filter(data[tbl]) ^ bit;
      update_contribution(data, tbl, m1, m2);
      data[tbl] ^= in;
    } else if (filter(data[tbl]) == bit) {
      data[++end] = data[tbl + 1];
      data[tbl + 1] = data[tbl] | 1;
      update_contribution(data, tbl, m1, m2);
      data[tbl++] ^= in;
      update_contribution(data, tbl, m1, m2);
      data[tbl] ^= in;
    } else {
      data[tbl--] = data[end--];
    }
  }
  return end;
}

static int old_recover(unsigned int odd[], int o_head, int o_tail, int oks, unsigned int even[],
                       int e_head, int e_tail, int eks, int rem, int s, RecoverCtx *ctx,
                       unsigned int in, int first_run) {
  int o, e, i;
  if (rem == -1) {
    for (e = e_head; e <= e_tail; ++e) {
      even[e] = (even[e] << 1) ^ evenparity32(even[e] & LF_POLY_EVEN) ^ (!!(in & 4));
      for (o = o_head; o <= o_tail; ++o, ++s) {
        struct Crypto1State temp = {0, 0};
        temp.even = odd[o];
        temp.odd = even[e] ^ evenparity32(odd[o] & LF_POLY_ODD);
        if (check_state(&temp, ctx)) {
          return -1;
        }
      }
    }
    return s;
  }
  if (first_run == 0) {
    for (i = 0; (i < 4) && (rem-- != 0); i++) {
      oks >>= 1;
      eks >>= 1;
      in >>= 2;
      o_tail =
          extend_table(odd, o_head, o_tail, oks & 1, LF_POLY_EVEN << 1 | 1, LF_POLY_ODD << 1, 0);
      if (o_head > o_tail)
        return s;
      e_tail =
          extend_table(even, e_head, e_tail, eks & 1, LF_POLY_ODD, LF_POLY_EVEN << 1 | 1, in & 3);
      if (e_head > e_tail)
        return s;
    }
  }
  first_run = 0;
  quicksort(odd, o_head, o_tail);
  quicksort(even, e_head, e_tail);
  while (o_tail >= o_head && e_tail >= e_head) {
    if (((odd[o_tail] ^ even[e_tail]) >> 24) == 0) {
      o_tail = binsearch(odd, o_head, o = o_tail);
      e_tail = binsearch(even, e_head, e = e_tail);
      s = old_recover(odd, o_tail--, o, oks, even, e_tail--, e, eks, rem, s, ctx, in, first_run);
      if (s == -1) {
        break;
      }
    } else if ((odd[o_tail] ^ 0x80000000) > (even[e_tail] ^ 0x80000000)) {
      o_tail = binsearch(odd, o_head, o_tail) - 1;
    } else {
      e_tail = binsearch(even, e_head, e_tail) - 1;
    }
  }
  return s;
}

static int calculate_msb_tables(int oks, int eks, int msb_round, RecoverCtx *ctx,
                                unsigned int *states_buffer, struct Msb *odd_msbs,
                                struct Msb *even_msbs, unsigned int *temp_states_odd,
                                unsigned int *temp_states_even, unsigned int in, uint32_t uid,
                                int MSB_LIMIT) {
  unsigned int msb_head = (MSB_LIMIT * msb_round);
  unsigned int msb_tail = (MSB_LIMIT * (msb_round + 1));
  int states_tail = 0, tail = 0;
  int i = 0, j = 0, semi_state = 0, found = 0;
  unsigned int msb = 0;
  in = ((in >> 16 & 0xff) | (in << 16) | (in & 0xff00)) << 1;

  memset(odd_msbs, 0, MSB_LIMIT * sizeof(struct Msb));
  memset(even_msbs, 0, MSB_LIMIT * sizeof(struct Msb));

  for (semi_state = 1 << 20; semi_state >= 0; semi_state--) {
    if ((semi_state & 0xFFF) == 0 && ctx->cb->should_stop && ctx->cb->should_stop(ctx->cb->user))
      return 0;

    if (semi_state % 65536 == 0) {
      float progress = (float)(1048576 - semi_state) / 1048576.0f * 100.0f;
      if (ctx->cb->progress)
        ctx->cb->progress((uint32_t)ctx->current_msb_round, (uint32_t)ctx->total_msb_rounds,
                          progress, uid, ctx->cb->user);
    }

    if (filter(semi_state) == (oks & 1)) {
      states_buffer[0] = semi_state;
      states_tail = state_loop(states_buffer, oks, CONST_M1_1, CONST_M2_1, 0, 0);

      for (i = states_tail; i >= 0; i--) {
        msb = states_buffer[i] >> 24;
        if ((msb >= msb_head) && (msb < msb_tail)) {
          found = 0;
          for (j = 0; j < odd_msbs[msb - msb_head].tail - 1; j++) {
            if (odd_msbs[msb - msb_head].states[j] == states_buffer[i]) {
              found = 1;
              break;
            }
          }

          if (!found) {
            tail = odd_msbs[msb - msb_head].tail++;
            odd_msbs[msb - msb_head].states[tail] = states_buffer[i];
          }
        }
      }
    }

    if (filter(semi_state) == (eks & 1)) {
      states_buffer[0] = semi_state;
      states_tail = state_loop(states_buffer, eks, CONST_M1_2, CONST_M2_2, in, 3);

      for (i = 0; i <= states_tail; i++) {
        msb = states_buffer[i] >> 24;
        if ((msb >= msb_head) && (msb < msb_tail)) {
          found = 0;

          for (j = 0; j < even_msbs[msb - msb_head].tail; j++) {
            if (even_msbs[msb - msb_head].states[j] == states_buffer[i]) {
              found = 1;
              break;
            }
          }

          if (!found) {
            tail = even_msbs[msb - msb_head].tail++;
            even_msbs[msb - msb_head].states[tail] = states_buffer[i];
          }
        }
      }
    }
  }

  oks >>= 12;
  eks >>= 12;

  for (i = 0; i < MSB_LIMIT; i++) {
    if (ctx->cb->should_stop && ctx->cb->should_stop(ctx->cb->user))
      return 0;

    memset(temp_states_even, 0, sizeof(unsigned int) * (1280));
    memset(temp_states_odd, 0, sizeof(unsigned int) * (1280));
    memcpy(temp_states_odd, odd_msbs[i].states, odd_msbs[i].tail * sizeof(unsigned int));
    memcpy(temp_states_even, even_msbs[i].states, even_msbs[i].tail * sizeof(unsigned int));

    int res = old_recover(temp_states_odd, 0, odd_msbs[i].tail, oks, temp_states_even, 0,
                          even_msbs[i].tail, eks, 3, 0, ctx, in >> 16, 1);
    if (res == -1) {
      return 1;
    }
  }

  return 0;
}

uint32_t prng_successor(uint32_t x, uint32_t n) {
  SWAPENDIAN(x);
  while (n--)
    x = x >> 1 | (x >> 16 ^ x >> 18 ^ x >> 19 ^ x >> 21) << 31;
  return SWAPENDIAN(x);
}

bool crapto1_recover(const CNonce *n, uint32_t ks2, uint32_t in, const CCallbacks *cb) {
  bool found = false;

  int MSB_LIMIT = MSB_LIMIT_BASE;
  if (n->attack == ATTACK_STATIC_ENCRYPTED) {
    MSB_LIMIT = MSB_LIMIT_BASE / 2;
  }

  struct Msb *odd_msbs = tls_odd_msbs;
  struct Msb *even_msbs = tls_even_msbs;
  unsigned int *temp_states_odd = tls_temp_states_odd;
  unsigned int *temp_states_even = tls_temp_states_even;
  unsigned int *states_buffer = tls_states_buffer;

  int oks = 0, eks = 0;
  int i = 0, msb = 0;

  for (i = 31; i >= 0; i -= 2) {
    oks = oks << 1 | BEBIT(ks2, i);
  }
  for (i = 30; i >= 0; i -= 2) {
    eks = eks << 1 | BEBIT(ks2, i);
  }

  RecoverCtx ctx;
  ctx.n = n;
  ctx.cb = cb;
  ctx.total_msb_rounds = 256 / MSB_LIMIT;
  ctx.current_msb_round = 0;

  for (msb = 0; msb <= ((256 / MSB_LIMIT) - 1); msb++) {
    ctx.current_msb_round = msb + 1;

    if (calculate_msb_tables(oks, eks, msb, &ctx, states_buffer, odd_msbs, even_msbs,
                             temp_states_odd, temp_states_even, in, n->uid, MSB_LIMIT)) {
      found = true;
      break;
    }
    if (cb->should_stop && cb->should_stop(cb->user)) {
      break;
    }

    if (cb->progress)
      cb->progress((uint32_t)ctx.current_msb_round, (uint32_t)ctx.total_msb_rounds, 100.0f, n->uid,
                   cb->user);
  }

  return found;
}

void crypto1_init(struct Crypto1State *state, uint64_t key) {
  if (state == NULL)
    return;
  state->odd = 0;
  state->even = 0;
  for (int i = 47; i > 0; i -= 2) {
    state->odd = state->odd << 1 | BIT(key, (i - 1) ^ 7);
    state->even = state->even << 1 | BIT(key, i ^ 7);
  }
}

struct Crypto1State *crypto1_create(uint64_t key) {
  struct Crypto1State *state = calloc(sizeof(*state), sizeof(uint8_t));
  if (!state)
    return NULL;
  crypto1_init(state, key);
  return state;
}

void crypto1_destroy(struct Crypto1State *state) { free(state); }

void crypto1_get_lfsr(struct Crypto1State *state, uint64_t *lfsr) {
  int i;
  for (*lfsr = 0, i = 23; i >= 0; --i) {
    *lfsr = *lfsr << 1 | BIT(state->odd, i ^ 3);
    *lfsr = *lfsr << 1 | BIT(state->even, i ^ 3);
  }
}

uint8_t crypto1_byte(struct Crypto1State *s, uint8_t in, int is_encrypted) {
  uint8_t ret = 0;
  ret |= crypto1_bit(s, BIT(in, 0), is_encrypted) << 0;
  ret |= crypto1_bit(s, BIT(in, 1), is_encrypted) << 1;
  ret |= crypto1_bit(s, BIT(in, 2), is_encrypted) << 2;
  ret |= crypto1_bit(s, BIT(in, 3), is_encrypted) << 3;
  ret |= crypto1_bit(s, BIT(in, 4), is_encrypted) << 4;
  ret |= crypto1_bit(s, BIT(in, 5), is_encrypted) << 5;
  ret |= crypto1_bit(s, BIT(in, 6), is_encrypted) << 6;
  ret |= crypto1_bit(s, BIT(in, 7), is_encrypted) << 7;
  return ret;
}

uint8_t lfsr_rollback_bit(struct Crypto1State *s, uint32_t in, int fb) {
  int out;
  uint8_t ret;
  uint32_t t;

  s->odd &= 0xffffff;
  t = s->odd;
  s->odd = s->even;
  s->even = t;

  out = s->even & 1;
  out ^= LF_POLY_EVEN & (s->even >>= 1);
  out ^= LF_POLY_ODD & s->odd;
  out ^= !!in;
  out ^= (ret = crypto1_filter(s->odd)) & (!!fb);

  s->even |= (evenparity32(out)) << 23;
  return ret;
}

uint8_t lfsr_rollback_byte(struct Crypto1State *s, uint32_t in, int fb) {
  uint8_t ret = 0;
  ret |= lfsr_rollback_bit(s, BIT(in, 7), fb) << 7;
  ret |= lfsr_rollback_bit(s, BIT(in, 6), fb) << 6;
  ret |= lfsr_rollback_bit(s, BIT(in, 5), fb) << 5;
  ret |= lfsr_rollback_bit(s, BIT(in, 4), fb) << 4;
  ret |= lfsr_rollback_bit(s, BIT(in, 3), fb) << 3;
  ret |= lfsr_rollback_bit(s, BIT(in, 2), fb) << 2;
  ret |= lfsr_rollback_bit(s, BIT(in, 1), fb) << 1;
  ret |= lfsr_rollback_bit(s, BIT(in, 0), fb) << 0;
  return ret;
}
