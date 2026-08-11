#include "hardnested_entry.h"

#include "hardnested.h"
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

static void (*g_line_cb)(const char *, void *) = NULL;
static void *g_line_user = NULL;

void PrintAndLogEx(const char *fmt, ...) {
  char buf[512];
  va_list ap;
  va_start(ap, fmt);
  vsnprintf(buf, sizeof(buf), fmt, ap);
  va_end(ap);

  if (g_line_cb != NULL) {
    g_line_cb(buf, g_line_user);
  } else {
    fputs(buf, stdout);
    fputc('\n', stdout);
  }
}

static void put_be32(uint8_t *p, uint32_t v) {
  p[0] = (uint8_t)(v >> 24);
  p[1] = (uint8_t)(v >> 16);
  p[2] = (uint8_t)(v >> 8);
  p[3] = (uint8_t)(v);
}

int hardnested_recover(uint32_t uid, uint8_t key_type, const HnNonce *nonces,
                       uint32_t count, const HnCallbacks *cb, uint64_t *out_key) {
  if (nonces == NULL || out_key == NULL || count < 2) {
    return 0;
  }

  if (cb != NULL) {
    g_line_cb = cb->line;
    g_line_user = cb->user;
  }

  uint32_t pairs = count / 2;
  uint32_t length = 6 + pairs * 9;
  uint8_t *blob = (uint8_t *)malloc(length);
  if (blob == NULL) {
    g_line_cb = NULL;
    g_line_user = NULL;
    return 0;
  }

  put_be32(blob, uid);
  blob[4] = 0;
  blob[5] = key_type;

  uint32_t pos = 6;
  for (uint32_t i = 0; i < pairs; i++) {
    const HnNonce *a = &nonces[2 * i];
    const HnNonce *b = &nonces[2 * i + 1];
    put_be32(blob + pos, a->nt_enc);
    put_be32(blob + pos + 4, b->nt_enc);
    blob[pos + 8] = (uint8_t)(((a->par & 0x0f) << 4) | (b->par & 0x0f));
    pos += 9;
  }

  uint64_t found = 0;
  int res = mfnestedhard(0, key_type, NULL, 0, 0, NULL, false, false, false,
                         &found, (char *)blob, length);
  free(blob);

  g_line_cb = NULL;
  g_line_user = NULL;

  if (res == 1) {
    *out_key = found;
    return 1;
  }
  return 0;
}
