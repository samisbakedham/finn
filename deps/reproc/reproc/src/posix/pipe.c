#include <pipe.h>

#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <stdlib.h>
#include <unistd.h>

REPROC_ERROR pipe_init(int *read,
                       struct pipe_options read_options,
                       int *write,
                       struct pipe_options write_options)
{
  assert(read);
  assert(write);

  int pipefd[2] = { -1, -1 };
  int r = -1;

  // On POSIX systems, by default file descriptors are inherited by child
  // processes when calling `exec`. To prevent unintended leaking of file
  // descriptors to child processes, POSIX provides a function `fcntl` which can
  // be used to set the `FD_CLOEXEC` flag which closes a file descriptor when
  // `exec` (or one of its variants) is called. However, using `fcntl`
  // introduces a race condition since any process spawned in another thread
  // after a file descriptor is created (for example using `pipe`) but before
  // `fcntl` is called to set `FD_CLOEXEC` on the file descriptor will still
  // inherit that file descriptor.

  // To get around this race condition we use the `pipe2` function when it
  // is available which takes the `O_CLOEXEC` flag as an argument. This ensures
  // the file descriptors of the created pipe are closed when `exec` is called.
  // If `pipe2` is not available we fall back to calling `fcntl` to set
  // `FD_CLOEXEC` immediately after creating a pipe.

#if defined(__APPLE__)
  r = pipe(pipefd);
  if (r < 0) {
    goto cleanup;
  }

  r = fcntl(pipefd[0], F_SETFD, FD_CLOEXEC);
  if (r < 0) {
    goto cleanup;
  }

  r = fcntl(pipefd[1], F_SETFD, FD_CLOEXEC);
  if (r < 0) {
    goto cleanup;
  }
#else
  r = pipe2(pipefd, O_CLOEXEC);
  if (r < 0) {
    goto cleanup;
  }
#endif

  r = fcntl(pipefd[0], F_SETFL, read_options.nonblocking ? O_NONBLOCK : 0);
  if (r < 0) {
    goto cleanup;
  }

  r = fcntl(pipefd[1], F_SETFL, write_options.nonblocking ? O_NONBLOCK : 0);
  if (r < 0) {
    goto cleanup;
  }

  *read = pipefd[0];
  *write = pipefd[1];

cleanup:
  if (r < 0) {
    handle_destroy(pipefd[0]);
    handle_destroy(pipefd[1]);
  }

  return r < 0 ? REPROC_ERROR_SYSTEM : REPROC_SUCCESS;
}

REPROC_ERROR
pipe_read(int pipe,
          uint8_t *buffer,
          unsigned int size,
          unsigned int *bytes_read)
{
  assert(buffer);
  assert(bytes_read);

  ssize_t r = -1;

  r = read(pipe, buffer, size);
  // `read` returns 0 to indicate the other end of the pipe was closed.
  if (r <= 0) {
    return r == 0 ? REPROC_ERROR_STREAM_CLOSED : REPROC_ERROR_SYSTEM;
  }

  *bytes_read = (unsigned int) r;

  return REPROC_SUCCESS;
}

static const int POLL_INFINITE = -1;

REPROC_ERROR pipe_write(int pipe,
                        const uint8_t *buffer,
                        unsigned int size,
                        unsigned int *bytes_written)
{
  assert(buffer);
  assert(bytes_written);

  struct pollfd pollfd = { .fd = pipe, .events = POLLOUT };
  ssize_t r = -1;

  r = poll(&pollfd, 1, POLL_INFINITE);
  if (r <= 0) {
    return REPROC_ERROR_SYSTEM;
  }

  assert(pollfd.revents & POLLOUT || pollfd.revents & POLLERR);

  r = write(pipe, buffer, size);
  if (r < 0) {
    switch (errno) {
      // `write` sets `errno` to `EPIPE` to indicate the other end of the pipe
      // was closed.
      case EPIPE:
        return REPROC_ERROR_STREAM_CLOSED;
      default:
        return REPROC_ERROR_SYSTEM;
    }
  }

  *bytes_written = (unsigned int) r;

  return REPROC_SUCCESS;
}

REPROC_ERROR
pipe_wait(const handle *pipes, unsigned int num_pipes, unsigned int *ready)
{
  assert(pipes);
  assert(ready);

  unsigned int i = 0;
  int r = -1;

  struct pollfd *pollfds = calloc(num_pipes, sizeof(struct pollfd));
  if (pollfds == NULL) {
    goto cleanup;
  }

  for (i = 0; i < num_pipes; i++) {
    pollfds[i].fd = pipes[i];
    pollfds[i].events = POLLIN;
  }

  r = poll(pollfds, num_pipes, POLL_INFINITE);
  if (r < 0) {
    goto cleanup;
  }

  for (i = 0; i < num_pipes; i++) {
    struct pollfd pollfd = pollfds[i];

    if (pollfd.revents & POLLIN || pollfd.revents & POLLERR) {
      *ready = i;
      goto cleanup;
    }
  }

cleanup:
  free(pollfds);

  return r < 0 ? REPROC_ERROR_SYSTEM
               : i == num_pipes ? REPROC_ERROR_STREAM_CLOSED : REPROC_SUCCESS;
}
