#include <pipe.h>

#include <macro.h>

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <windows.h>

enum {
  PIPE_BUFFER_SIZE = 65536,
  PIPE_SINGLE_INSTANCE = 1,
  PIPE_NO_TIMEOUT = 0,
  FILE_NO_SHARE = 0,
  FILE_NO_TEMPLATE = 0
};

static SECURITY_ATTRIBUTES HANDLE_DO_NOT_INHERIT = {
  .nLength = sizeof(SECURITY_ATTRIBUTES),
  .bInheritHandle = false,
  .lpSecurityDescriptor = NULL
};

static THREAD_LOCAL unsigned long pipe_serial_number = 0;

REPROC_ERROR
pipe_init(HANDLE *read,
          struct pipe_options read_options,
          HANDLE *write,
          struct pipe_options write_options)
{
  assert(read);
  assert(write);

  // Windows anonymous pipes don't support overlapped I/O so we use named pipes
  // instead. This code has been adapted from
  // https://stackoverflow.com/a/419736/11900641.
  //
  // Copyright (c) 1997  Microsoft Corporation

  // Thread-safe unique name for the named pipe.
  char name[MAX_PATH];
  sprintf(name, "\\\\.\\Pipe\\RemoteExeAnon.%08lx.%08lx.%08lx",
          GetCurrentProcessId(), GetCurrentThreadId(), pipe_serial_number++);

  HANDLE pipe_handles[2] = { HANDLE_INVALID, HANDLE_INVALID };

  DWORD read_mode = read_options.nonblocking ? FILE_FLAG_OVERLAPPED : 0;
  DWORD write_mode = write_options.nonblocking ? FILE_FLAG_OVERLAPPED : 0;

  SECURITY_ATTRIBUTES security = { .nLength = sizeof(SECURITY_ATTRIBUTES),
                                   .lpSecurityDescriptor = NULL };

  security.bInheritHandle = read_options.inherit;

  pipe_handles[0] = CreateNamedPipeA(name, PIPE_ACCESS_INBOUND | read_mode,
                                     PIPE_TYPE_BYTE | PIPE_WAIT,
                                     PIPE_SINGLE_INSTANCE, PIPE_BUFFER_SIZE,
                                     PIPE_BUFFER_SIZE, PIPE_NO_TIMEOUT,
                                     &security);
  if (pipe_handles[0] == INVALID_HANDLE_VALUE) {
    goto cleanup;
  }

  security.bInheritHandle = write_options.inherit;

  pipe_handles[1] = CreateFileA(name, GENERIC_WRITE, FILE_NO_SHARE, &security,
                                OPEN_EXISTING,
                                FILE_ATTRIBUTE_NORMAL | write_mode,
                                (HANDLE) FILE_NO_TEMPLATE);
  if (pipe_handles[1] == INVALID_HANDLE_VALUE) {
    goto cleanup;
  }

  *read = pipe_handles[0];
  *write = pipe_handles[1];

cleanup:
  if (pipe_handles[1] == INVALID_HANDLE_VALUE) {
    handle_destroy(pipe_handles[0]);
    handle_destroy(pipe_handles[1]);
  }

  return pipe_handles[1] == INVALID_HANDLE_VALUE ? REPROC_ERROR_SYSTEM
                                                 : REPROC_SUCCESS;
}

REPROC_ERROR pipe_read(HANDLE pipe,
                       uint8_t *buffer,
                       unsigned int size,
                       unsigned int *bytes_read)
{
  assert(pipe);
  assert(buffer);
  assert(bytes_read);

  BOOL r = 0;

  OVERLAPPED overlapped = { 0 };
  overlapped.hEvent = CreateEvent(&HANDLE_DO_NOT_INHERIT, true, false, NULL);
  if (overlapped.hEvent == NULL) {
    goto cleanup;
  }

  r = ReadFile(pipe, buffer, size, NULL, &overlapped);
  if (r == 0 && GetLastError() != ERROR_IO_PENDING) {
    goto cleanup;
  }

  // The cast is safe since `DWORD` is a typedef to `unsigned int` on Windows.
  r = GetOverlappedResult(pipe, &overlapped, (LPDWORD) bytes_read, true);
  if (r == 0) {
    goto cleanup;
  }

cleanup:
  handle_destroy(overlapped.hEvent);

  return r > 0
             ? REPROC_SUCCESS
             : GetLastError() == ERROR_BROKEN_PIPE ? REPROC_ERROR_STREAM_CLOSED
                                                   : REPROC_ERROR_SYSTEM;
}

REPROC_ERROR pipe_write(HANDLE pipe,
                        const uint8_t *buffer,
                        unsigned int size,
                        unsigned int *bytes_written)
{
  assert(pipe);
  assert(buffer);
  assert(bytes_written);

  BOOL r = 0;

  OVERLAPPED overlapped = { 0 };
  overlapped.hEvent = CreateEvent(&HANDLE_DO_NOT_INHERIT, true, false, NULL);
  if (overlapped.hEvent == NULL) {
    goto cleanup;
  }

  r = WriteFile(pipe, buffer, size, NULL, &overlapped);
  if (r == 0 && GetLastError() != ERROR_IO_PENDING) {
    goto cleanup;
  }

  // The cast is safe since `DWORD` is a typedef to `unsigned int` on Windows.
  r = GetOverlappedResult(pipe, &overlapped, (LPDWORD) bytes_written, true);
  if (r == 0) {
    goto cleanup;
  }

cleanup:
  handle_destroy(overlapped.hEvent);

  return r > 0
             ? REPROC_SUCCESS
             : GetLastError() == ERROR_BROKEN_PIPE ? REPROC_ERROR_STREAM_CLOSED
                                                   : REPROC_ERROR_SYSTEM;
}

REPROC_ERROR
pipe_wait(const handle *pipes, unsigned int num_pipes, unsigned int *ready)
{
  assert(ready);

  OVERLAPPED *overlapped = NULL;
  HANDLE *events = NULL;
  DWORD num_reads = 0;
  DWORD r = 0;

  // `BOOL` is either 0 or 1 even though it is defined as `int` so all casts of
  // functions returning `BOOL` to `DWORD` in this function are safe.

  overlapped = calloc(num_pipes, sizeof(OVERLAPPED));
  if (overlapped == NULL) {
    goto cleanup;
  }

  events = calloc(num_pipes, sizeof(HANDLE));
  if (events == NULL) {
    goto cleanup;
  }

  // We emulate POSIX `poll` by issuing overlapped zero-sized reads and waiting
  // for the first one to complete. This approach is inspired by the CPython
  // multiprocessing module `wait` implementation:
  // https://github.com/python/cpython/blob/10ecbadb799ddf3393d1fc80119a3db14724d381/Lib/multiprocessing/connection.py#L826

  for (DWORD i = 0; i < num_pipes; i++) {
    overlapped[i].hEvent = CreateEvent(&HANDLE_DO_NOT_INHERIT, true, false,
                                       NULL);
    if (overlapped[i].hEvent == NULL) {
      goto cleanup;
    }

    r = (DWORD) ReadFile(pipes[i], (uint8_t[]){ 0 }, 0, NULL, &overlapped[i]);
    if (r == 0 && GetLastError() != ERROR_IO_PENDING &&
        GetLastError() != ERROR_BROKEN_PIPE) {
      goto cleanup;
    }

    if (r != 0 || GetLastError() == ERROR_IO_PENDING) {
      num_reads++;
    }

    events[i] = overlapped[i].hEvent;
  }

  if (num_reads == 0) {
    // All streams have been closed.
    r = 1;
    goto cleanup;
  }

  r = WaitForMultipleObjects(num_pipes, events, false, INFINITE);

  // We don't expect `WAIT_ABANDONED_0` or `WAIT_TIMEOUT` to be valid here.
  assert(r < WAIT_ABANDONED_0);
  assert(r != WAIT_TIMEOUT);

  if (r == WAIT_FAILED) {
    goto cleanup;
  }

  assert(r < num_pipes);

  // Map the signaled event back to its corresponding handle.
  *ready = r;
  r = 1;

cleanup:
  // If a memory allocation failed, we don't have to do any other cleanup.
  if (overlapped == NULL || events == NULL) {
    free(overlapped);
    free(events);
    return REPROC_ERROR_SYSTEM;
  }

  // Because we continue cleaning up even when an error occurs during cleanup,
  // we have to save the first error that occurs because `GetLastError()` might
  // get overridden with another error during further cleanup. When we're done,
  // we reset `GetLastError()` to the first error that occurred that we couldn't
  // handle.
  DWORD first_system_error = GetLastError();
  BOOL k = 0;

  for (DWORD i = 0; i < num_pipes; i++) {

    // Cancel any remaining zero-sized reads that we queued if they have not yet
    // completed.

    k = CancelIoEx(pipes[i], &overlapped[i]);
    if (k == 0) {
      if (GetLastError() != ERROR_NOT_FOUND && first_system_error == 0) {
        first_system_error = GetLastError();
      }

      continue;
    }

    // `CancelIoEx` only requests cancellation. We use `GetOverlappedResult` to
    // wait until the read is actually cancelled.

    DWORD bytes_transferred = 0;
    k = GetOverlappedResult(pipes[i], &overlapped[i], &bytes_transferred, true);
    if (k == 0 && GetLastError() != ERROR_OPERATION_ABORTED &&
        GetLastError() != ERROR_BROKEN_PIPE && first_system_error == 0) {
      first_system_error = GetLastError();
    }
  }

  for (DWORD i = 0; i < num_pipes; i++) {
    handle_destroy(overlapped[i].hEvent);
  }

  free(overlapped);
  free(events);

  if (first_system_error > 0) {
    SetLastError(first_system_error);
  }

  return r == 0 ? REPROC_ERROR_SYSTEM
                : num_reads == 0 ? REPROC_ERROR_STREAM_CLOSED : REPROC_SUCCESS;
}
