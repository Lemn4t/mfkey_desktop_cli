#ifdef _WIN32

#include "pthread.h"
#include <process.h>
#include <stdint.h>
#include <stdlib.h>

struct mfkey_thread_start {
  void *(*fn)(void *);
  void *arg;
};

static unsigned __stdcall mfkey_thread_trampoline(void *p) {
  struct mfkey_thread_start s = *(struct mfkey_thread_start *)p;
  free(p);
  s.fn(s.arg);
  return 0;
}

int pthread_create(pthread_t *thread, const pthread_attr_t *attr,
                   void *(*start_routine)(void *), void *arg) {
  (void)attr;
  struct mfkey_thread_start *s = malloc(sizeof(*s));
  if (!s) {
    return -1;
  }
  s->fn = start_routine;
  s->arg = arg;
  uintptr_t h = _beginthreadex(NULL, 0, mfkey_thread_trampoline, s, 0, NULL);
  if (h == 0) {
    free(s);
    return -1;
  }
  *thread = (HANDLE)h;
  return 0;
}

int pthread_join(pthread_t thread, void **retval) {
  (void)retval;
  WaitForSingleObject(thread, INFINITE);
  CloseHandle(thread);
  return 0;
}

int pthread_mutex_init(pthread_mutex_t *m, const pthread_mutexattr_t *attr) {
  (void)attr;
  InitializeSRWLock(m);
  return 0;
}

int pthread_mutex_destroy(pthread_mutex_t *m) {
  (void)m;
  return 0;
}

int pthread_mutex_lock(pthread_mutex_t *m) {
  AcquireSRWLockExclusive(m);
  return 0;
}

int pthread_mutex_unlock(pthread_mutex_t *m) {
  ReleaseSRWLockExclusive(m);
  return 0;
}

#endif
