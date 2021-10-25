#pragma once

#include <reproc++/arguments.hpp>
#include <reproc++/environment.hpp>
#include <reproc++/error.hpp>
#include <reproc++/export.hpp>

#include <chrono>
#include <cstdint>
#include <memory>

// Forward declare `reproc_t` so we don't have to include reproc.h in the
// header.
struct reproc_t;

/*! The `reproc` namespace wraps all reproc++ declarations. `process` wraps
reproc's API inside a C++ class. `error` improves on `REPROC_ERROR` by
integrating with C++'s `std::error_code` error handling mechanism. To avoid
exposing reproc's API when using reproc++ all the other structs, enums and
constants of reproc have a replacement in reproc++ as well. */
namespace reproc {

using milliseconds = std::chrono::duration<unsigned int, std::milli>;

/*! `REPROC_REDIRECT` */
enum class redirect { pipe, inherit, discard };

/*! `REPROC_STOP` */
enum class stop { noop, wait, terminate, kill };

struct stop_action {
  stop action;
  milliseconds timeout;
};

struct stop_actions {
  stop_action first;
  stop_action second;
  stop_action third;
};

/*! `reproc_options` */
struct options {
  class environment environment;
  const char *working_directory = nullptr;

  struct {
    redirect in;
    redirect out;
    redirect err;
  } redirect = {};

  struct stop_actions stop_actions = {};
};

/*! `REPROC_STREAM` */
enum class reproc_stream { in, out, err };

/*! `REPROC_INFINITE` */
REPROCXX_EXPORT extern const milliseconds infinite;

/*! Improves on reproc's API by wrapping it in a class. Aside from methods that
mimick reproc's API it also adds configurable RAII and several methods that
reduce the boilerplate required to use reproc from idiomatic C++ code. */
class process {

public:
  /*!
  Allocates memory for reproc's `reproc_t` struct.

  The arguments are passed to `stop` in the destructor if the process is still
  running by then.

  Example:

  ```c++
  process example(stop::wait, 10000, stop::terminate, 5000);
  ```

  If the child process is still running when example's destructor is called, it
  will first wait ten seconds for the child process to exit on its own before
  sending `SIGTERM` (POSIX) or `CTRL-BREAK` (Windows) and waiting five more
  seconds for the child process to exit.

  The default arguments instruct the destructor to wait indefinitely for the
  child process to exit.
  */
  REPROCXX_EXPORT process();

  /*! `reproc_destroy` */
  REPROCXX_EXPORT ~process() noexcept = default;

  // Enforce unique ownership of child processes.
  REPROCXX_EXPORT process(process &&other) noexcept = default;
  REPROCXX_EXPORT process &operator=(process &&other) noexcept = default;

  /*! `reproc_start` */
  REPROCXX_EXPORT std::error_code start(const arguments &arguments,
                                        const options &options = {}) noexcept;

  /*! `reproc_read` */
  REPROCXX_EXPORT std::error_code read(reproc_stream *stream,
                                       uint8_t *buffer,
                                       unsigned int size,
                                       unsigned int *bytes_read) noexcept;

  /*!
  Calls `read` on `stream` until `parser` returns false or an error occurs.
  `parser` receives the output after each read.

  `parser` is always called once with an empty buffer to give the parser the
  chance to process all output from the previous call to `parse` one by one.

  `Parser` expects the following signature:

  ```c++
  bool parser(stream stream, const uint8_t *buffer, unsigned int size);
  ```
  */
  template <typename Parser>
  std::error_code parse(Parser &&parser);

  /*!
  `parse` but `stream_closed` is not treated as an error. `Sink` expects the
  same signature as `Parser` in `parse`.

  For examples of sinks, see `sink.hpp`.
  */
  template <typename Sink>
  std::error_code drain(Sink &&sink);

  /*! `reproc_write` */
  REPROCXX_EXPORT std::error_code write(const uint8_t *buffer,
                                        unsigned int size) noexcept;

  /*! `reproc_close` */
  REPROCXX_EXPORT void close(reproc_stream stream) noexcept;

  /*! `reproc_running` */
  REPROCXX_EXPORT bool running() noexcept;

  /*! `reproc_wait` */
  REPROCXX_EXPORT std::error_code wait(milliseconds timeout) noexcept;

  /*! `reproc_terminate` */
  REPROCXX_EXPORT std::error_code terminate() noexcept;

  /*! `reproc_kill` */
  REPROCXX_EXPORT std::error_code kill() noexcept;

  /*! `reproc_stop` */
  REPROCXX_EXPORT std::error_code
  stop(stop_actions stop_actions) noexcept;

  /*! `reproc_exit_status` */
  REPROCXX_EXPORT int exit_status() noexcept;

private:
  std::unique_ptr<reproc_t, void (*)(reproc_t *)> process_;
};

template <typename Parser>
std::error_code process::parse(Parser &&parser)
{
  // We can't use compound literals in C++ to pass the initial value to `parser`
  // so we use a constexpr value instead.
  static constexpr uint8_t initial = 0;

  // A single call to `read` might contain multiple messages. By always calling
  // `parser` once with no data before reading, we give it the chance to process
  // all previous output one by one before reading from the child process again.
  if (!parser(reproc_stream::in, &initial, 0)) {
    return {};
  }

  uint8_t buffer[4096]; // NOLINT
  std::error_code ec;

  while (true) {
    reproc_stream stream = {};
    unsigned int bytes_read = 0;
    ec = read(&stream, buffer, sizeof(buffer), &bytes_read);
    if (ec) {
      break;
    }

    // `parser` returns false to tell us to stop reading.
    if (!parser(stream, buffer, bytes_read)) {
      break;
    }
  }

  return ec;
}

template <typename Sink>
std::error_code process::drain(Sink &&sink)
{
  std::error_code ec = parse(std::forward<Sink>(sink));

  return ec == error::stream_closed ? error::success : ec;
}

} // namespace reproc
