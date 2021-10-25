#include <reproc++/reproc.hpp>
#include <reproc++/sink.hpp>

#include <future>
#include <iostream>
#include <mutex>
#include <string>

static int fail(std::error_code ec)
{
  std::cerr << ec.message();
  return 1;
}

// The background example reads the output of a child process in a background
// thread and shows how to access the current output in the main thread while
// the background thread is still running.

// Like the forward example it forwards its arguments to a child process and
// prints the child process output on stdout.
int main(int argc, char *argv[])
{
  if (argc <= 1) {
    std::cerr << "No arguments provided. Example usage: "
              << "./background cmake --help";
    return 1;
  }

  reproc::process process;

  reproc::stop_actions stop_actions = {
    { reproc::stop::terminate, reproc::milliseconds(5000) },
    { reproc::stop::kill, reproc::milliseconds(2000) },
    {}
  };

  reproc::options options;
  options.stop_actions = stop_actions;

  std::error_code ec = process.start(argv + 1, options);

  if (ec == std::errc::no_such_file_or_directory) {
    std::cerr << "Program not found. Make sure it's available from the PATH.";
    return 1;
  } else if (ec) {
    return fail(ec);
  }

  // We need a mutex along with `output` to prevent the main thread and
  // background thread from modifying `output` at the same time (`std::string`
  // is not thread safe).
  std::string output;
  std::mutex mutex;

  auto drain_async = std::async(std::launch::async, [&process, &output,
                                                     &mutex]() {
    // `sink::thread_safe::string` locks a given mutex before appending to the
    // given string(s), allowing working with the string(s) across multiple
    // threads if the mutex is locked in the other threads as well.
    reproc::sink::thread_safe::string sink(output, output, mutex);
    return process.drain(sink);
  });

  // Show new output every 2 seconds.
  while (drain_async.wait_for(std::chrono::seconds(2)) !=
         std::future_status::ready) {
    std::lock_guard<std::mutex> lock(mutex);
    std::cout << output;
    // Clear output that's already been flushed to `std::cout`.
    output.clear();
  }

  // Flush any remaining output of `process`.
  std::cout << output;

  // Check if any errors occurred in the background thread.
  ec = drain_async.get();
  if (ec) {
    return fail(ec);
  }

  ec = process.stop(stop_actions);
  if (ec) {
    return fail(ec);
  }

  return process.exit_status();
}
