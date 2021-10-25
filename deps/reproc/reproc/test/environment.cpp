#include <doctest.h>

#include <reproc/reproc.h>
#include <reproc/sink.h>

#include <array>
#include <iterator>
#include <sstream>

TEST_CASE("environment")
{
  reproc_t *process = reproc_new();
  REQUIRE(process);

  REPROC_ERROR error = REPROC_SUCCESS;
  INFO(reproc_error_system());
  INFO(reproc_error_string(error));

  std::array<const char *, 2> argv = { RESOURCE_DIRECTORY "/environment",
                                       nullptr };
  std::array<const char *, 3> envp = { "IP=127.0.0.1", "PORT=8080", nullptr };

  reproc_options options = {};
  options.environment = envp.data();

  error = reproc_start(process, argv.data(), options);
  REQUIRE(!error);

  char *output = nullptr;
  error = reproc_drain(process, reproc_sink_string, &output);
  REQUIRE(!error);
  REQUIRE(output != nullptr);

  error = reproc_wait(process, REPROC_INFINITE);
  REQUIRE(!error);
  REQUIRE(reproc_exit_status(process) == 0);

  reproc_destroy(process);

  std::ostringstream concatenated;
  std::copy(envp.begin(), envp.end() - 1,
            std::ostream_iterator<const char *>(concatenated, ""));

  REQUIRE(output == concatenated.str());

  free(output);
}
