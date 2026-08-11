#ifndef MFKEY_PTHREAD_SHIM_H
#define MFKEY_PTHREAD_SHIM_H

#ifdef _WIN32

#include <windows.h>

typedef HANDLE pthread_t;
typedef SRWLOCK pthread_mutex_t;
typedef void pthread_attr_t;
typedef void pthread_mutexattr_t;

#define PTHREAD_MUTEX_INITIALIZER SRWLOCK_INIT

#ifdef __cplusplus
extern "C" {
#endif

int pthread_create(pthread_t *thread, const pthread_attr_t *attr,
                   void *(*start_routine)(void *), void *arg);
int pthread_join(pthread_t thread, void **retval);
int pthread_mutex_init(pthread_mutex_t *m, const pthread_mutexattr_t *attr);
int pthread_mutex_destroy(pthread_mutex_t *m);
int pthread_mutex_lock(pthread_mutex_t *m);
int pthread_mutex_unlock(pthread_mutex_t *m);

#ifdef __cplusplus
}
#endif

#endif

#endif
